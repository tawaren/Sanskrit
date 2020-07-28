#![no_std]
#![feature(nll)]

#[macro_use]
extern crate arrayref;
extern crate byteorder;
extern crate num_traits;
//extern crate ed25519_dalek;
//extern crate sha2;
extern crate sanskrit_interpreter;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;
extern crate alloc;


use sanskrit_common::store::{Store, StorageClass, store_hash, ChangeReport};
use sanskrit_common::errors::*;
use sanskrit_common::encoding::{Parser, Serializer};
use model::{Transaction, ParamRef, RetType, TransactionBundle, ParamMode};
use sanskrit_common::model::{Hash, SlicePtr, Ptr};
//use ed25519_dalek::*;
//use sha2::{Sha512};
use sanskrit_common::arena::*;
use sanskrit_interpreter::interpreter::{Frame, ExecutionContext};
use sanskrit_common::hashing::{HashingDomain, Hasher};
use sanskrit_interpreter::model::{Entry, Adt, TransactionDescriptor, TxTParam, TxTReturn, ValueSchema, RuntimeType};
use alloc::vec::Vec;
use alloc::format;

use alloc::collections::{BTreeMap, BTreeSet};
use system::{is_entry, is_context};
use core::convert::TryInto;
use core::ops::Deref;
use core::cell::RefCell;

pub mod model;
mod system;

pub const STORE_MODE: bool = true;

//Todo: The gas & entry & volume Accounting for storage is clunky
//      We need a better solution

//Todo: We need a store Entry Limit to prevent attacks when storing to much
//      This is needed as we account after the fact
//      With that we can check that we have enough for the worst case


pub const ENCODING_COST: DataProcessingCost = DataProcessingCost {
    cost_multiplier: 1,
    cost_divider: 5
};

pub const CONFIG: Configuration = Configuration {
    max_stack_depth:2048,
    max_frame_depth:512,
    max_heap_size:512 * 1024,
    max_structural_dept:64,
    max_transaction_memory:512 * 1024,
    max_transaction_size: 128 * 1024,
    return_stack: 256,
    parsing_cost: ENCODING_COST,
    store_cost: 2000,
    load_cost: 200,
};

impl Configuration {
    pub const fn calc_heap_size(&self, virt_factor:usize) -> usize {
        Heap::elems::<Entry>(self.max_stack_depth)
        + Heap::elems::<Frame>(self.max_stack_depth)
        + Heap::elems::<Entry>(self.return_stack)
        + (self.max_heap_size* virt_factor)
        + (self.max_transaction_memory * virt_factor)
    }
}

pub struct Configuration {
    pub max_stack_depth:usize,
    pub max_frame_depth:usize,
    pub max_heap_size:usize,
    pub max_structural_dept: usize,
    pub max_transaction_memory: usize,
    pub max_transaction_size: usize,
    pub return_stack: usize,
    pub load_cost: u64,
    pub store_cost: u64,
    pub parsing_cost: DataProcessingCost,
}

pub struct DataProcessingCost {
    pub cost_multiplier:u64,
    pub cost_divider:u64,
}

#[derive(Copy, Clone)]
pub enum ParamCache<'a> {
    Unloaded,
    Loaded(Entry<'a>, Hash)
}

//A struct holding context information of the current transaction
pub struct ExecutionEnvironment<'a, 'b, 'c, S:Store> {
    store:&'a S,

    txt_bundle:TransactionBundle<'b>,
    parameter_heap:&'b VirtualHeapArena<'c>,

    max_desc_alloc:VirtualHeapArena<'c>,
    structural_arena:HeapArena<'c>,
    runtime_heap:VirtualHeapArena<'c>,
    required_gas_prepay:u64,
    required_entry_storage_prepay:i64,
    //used for local accounting
    available_gas:u64,
    available_entry_loaded:u32,
    available_entry_created:u32,
    available_entry_deleted:u32,
    //Caches to parse each param just once
    entry_cache:RefCell<Vec<ParamCache<'b>>>,
    literal_cache:RefCell<Vec<ParamCache<'b>>>,
    witness_cache:RefCell<Vec<ParamCache<'b>>>,
}

pub struct TxTStats {
    pub loaded:u8,
    pub created:u8,
    pub deleted:u8,
    pub gas: u32,
}

pub trait Logger {
    fn log<'a>(&mut self, data:Vec<u8>);
}


//TODO: Make cmplete rework of the fas stuff and see if with the witnesses we can get it in order
//      In storage / provider mode do not execute payment txts (for now)
//

//Executes a transaction
pub fn execute<S:Store, L:Logger>(store:&S, txt_bundle_data:&[u8], block_no:u64, heap:&Heap, logger:&mut L) -> Result<()> {
    //Check that it is inside limit
    if txt_bundle_data.len() > CONFIG.max_transaction_size { return error(||"Transaction Bundle to big")}
    //Static allocations (could be done once)
    // A buffer to parse the transaction and load values from store
    let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_transaction_memory); //todo: will be static conf later (or block consensus)
    //Parse the transaction
    let txt_bundle:TransactionBundle = Parser::parse_fully(txt_bundle_data, CONFIG.max_structural_dept, &txt_bundle_alloc)?;

    //Calculate the payment information
    //Starts with the parsing costs which are already done mostly but this is inevitable (a miner could have a size limit)
    //This includes encoding the parameter witnesses
    //todo: only thin missing is the cost for loading & parsing the Transactions themself
    //      but these are predictable and could be set per section and then checked
    let mut required_gas_prepay = (CONFIG.parsing_cost.cost_multiplier * (txt_bundle_data.len() as u64))/ CONFIG.parsing_cost.cost_divider;
    let mut required_entry_storage_prepay = 0;


    let mut change_in_stored_entries = 0;
    for txt_section in txt_bundle.sections.iter() {
        required_gas_prepay += txt_section.gas_limit as u64;
        let entries_diff = (txt_section.entries_created as i64) - (txt_section.entries_deleted as i64);
        let entry_loads = txt_section.entries_loaded;
        let entry_stores = txt_section.entries_deleted + txt_section.entries_created;
        required_gas_prepay += CONFIG.load_cost * (entry_loads as u64);
        required_gas_prepay += CONFIG.store_cost * (entry_stores as u64);
        change_in_stored_entries += entries_diff;
        //This is necessary as we can abort after each section so we have to prefond for the worst case
        if required_entry_storage_prepay < change_in_stored_entries {
            required_entry_storage_prepay = change_in_stored_entries
        }

    }

    //Todo: we need to have a witness limit in the non witness part to make sure that a bad miner does not manipulate the witnesses to make the sender pay for more
    //      can he do this anyway? what happens if the second txt has an invalid witness?
    //      shall we parse witnesses upfront? to protect txt sender (but then teh miner has higher risk)
    //       Options?
    let witness_size = txt_bundle.witness.iter().map(|w|w.len() + 2).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
    let store_witness_size = txt_bundle.store_witness.iter().map(|w|w.map_or(0,|d|d.len()+2)+1).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
    let witness_start = txt_bundle_data.len() - witness_size - store_witness_size;

    let full_bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data);
    let bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data[..witness_start]);

    //Create Allocator
    //create heaps: based on bundle input
    let mut structural_arena = heap.new_arena(
        Heap::elems::<Entry>(txt_bundle.stack_elem_limit as usize)
            + Heap::elems::<Frame>(txt_bundle.stack_frame_limit as usize)
            + Heap::elems::<Entry>(CONFIG.return_stack)
    );

    let mut max_desc_alloc = heap.new_virtual_arena(txt_bundle.transaction_storage_heap as usize);
    let mut parameter_heap = heap.new_virtual_arena(txt_bundle.param_heap_limit as usize);
    let mut runtime_heap = heap.new_virtual_arena(txt_bundle.runtime_heap_limit as usize);

    let entry_cache = RefCell::new(alloc::vec::from_elem(ParamCache::Unloaded, txt_bundle.store_witness.len()));
    let literal_cache = RefCell::new(alloc::vec::from_elem(ParamCache::Unloaded, txt_bundle.literal.len()));
    let witness_cache = RefCell::new(alloc::vec::from_elem(ParamCache::Unloaded,txt_bundle.witness.len()));

    let mut exec_env = ExecutionEnvironment {
        store,
        max_desc_alloc,
        txt_bundle,
        structural_arena,
        parameter_heap: &parameter_heap,
        runtime_heap,
        required_gas_prepay,
        required_entry_storage_prepay,
        //todo: whe can use these i reverse in storage provider mode
        //  Calc them instead of using them as limit
        available_gas:0,            //will be set for each section
        available_entry_loaded:0,   //will be set for each section
        available_entry_created:0,  //will be set for each section
        available_entry_deleted:0,  //will be set for each section
        entry_cache,
        literal_cache,
        witness_cache
    };

    if exec_env.txt_bundle.store_witness.len() != exec_env.txt_bundle.stored.len() { return error(||"Not enough witnesses for stored values") }

    for txt_section in exec_env.txt_bundle.sections.iter() {
        exec_env.available_gas = txt_section.gas_limit;
        exec_env.available_entry_loaded = txt_section.entries_loaded;
        exec_env.available_entry_created = txt_section.entries_created;
        exec_env.available_entry_deleted = txt_section.entries_deleted;

        for txt in txt_section.txts.iter() {
            match execute_transaction( &exec_env, txt, &full_bundle_hash,  &bundle_hash, logger ) {
                Ok(stats) => {
                    //charge for the used gas
                    exec_env.available_gas -= stats.gas as u64;
                    exec_env.available_entry_loaded -= stats.loaded as u32;
                    exec_env.available_entry_created -= stats.created as u32;
                    exec_env.available_entry_deleted -= stats.deleted as u32;
                },
                Err(err) => {
                    rollback_store(&exec_env)?;
                    return Err(err);
                }
            };

            //release all the memory so it does not leak into the next transaction
            exec_env.structural_arena = exec_env.structural_arena.reuse();
            exec_env.max_desc_alloc = exec_env.max_desc_alloc.reuse();
            exec_env.runtime_heap = exec_env.runtime_heap.reuse();
        }

        //commit
        commit_store(&exec_env)?;
    }

    Ok(())
}



fn execute_transaction<S:Store, L:Logger>(env:&ExecutionEnvironment<S>,txt:&Transaction,full_hash:&Hash,txt_hash:&Hash,logger:&mut L) -> Result<TxTStats>{

    //Prepare all the Memory
    if env.txt_bundle.descriptors.len() <= txt.txt_desc as usize { return error(||"Descriptor index out of range")  }
    let target = &env.txt_bundle.descriptors[txt.txt_desc as usize];
    //todo: pre calculate this: how to gas charge this
    let txt_desc:TransactionDescriptor = env.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, &env.max_desc_alloc)?;

    if txt_desc.gas_cost as u64 > env.available_gas {return error(||"Bundle has not reserved enough gas")}
    if txt_desc.max_frames > env.txt_bundle.stack_frame_limit {return error(||"Bundle has not reserved enough frame space")}
    if txt_desc.max_stack > env.txt_bundle.stack_elem_limit {return error(||"Bundle has not reserved enough stack space")}
    if txt_desc.max_mem > env.txt_bundle.runtime_heap_limit {return error(||"Bundle has not reserved enough heap space")}

    let mut interpreter_stack = env.structural_arena.alloc_stack::<Entry>(txt_desc.max_stack as usize);
    let mut frame_stack = env.structural_arena.alloc_stack::<Frame>(txt_desc.max_frames as usize);
    let mut return_stack = env.structural_arena.alloc_stack::<Entry>(CONFIG.return_stack);

    //push everything required onto the stack
    let mut lock_set = BTreeSet::new();
    let mut deletes = Vec::with_capacity(txt_desc.params.len());

    let mut strats = TxTStats {
        loaded: 0,
        created: 0,
        deleted: 0,
        gas: txt_desc.gas_cost
    };

    for (p,p_typ) in txt_desc.params.iter().zip(txt.params.iter()) {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                strats.loaded+=1;
                strats.deleted+=1;
                if strats.loaded as u32 > env.available_entry_loaded  {return error(||"Bundle has not reserved enough entry loads")}
                if strats.deleted as u32 > env.available_entry_deleted  {return error(||"Bundle has not reserved enough entry deletes")}
                if !p.consumes && !p.drop { return error(||"A owned store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                //We delete at end so others can copy and in case it produces an error it must still be their
                deletes.push(hash);
                let data = load_from_store(env,*index, hash,*p)?;
                interpreter_stack.push(data)?;
            }
            ParamRef::Load(ParamMode::Copy, index) => {
                strats.loaded+=1;
                if strats.loaded as u32 > env.available_entry_loaded  {return error(||"Bundle has not reserved enough entry loads")}
                if !p.copy { return error(||"A Copied store value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A Copied store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                let data = load_from_store(env,*index, hash,*p)?;
                interpreter_stack.push(data)?;
            },
            ParamRef::Load(ParamMode::Borrow, index) => {
                strats.loaded+=1;
                if strats.loaded as u32 > env.available_entry_loaded  {return error(||"Bundle has not reserved enough entry loads")}
                if p.consumes { return error(||"A Borrowed store value can not be consumed") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                let data = load_from_store(env,*index, hash,*p)?;
                interpreter_stack.push(data)?;
            },

            ParamRef::Provided => {
                if is_context(p.typ) {
                    interpreter_stack.push(create_ctx(&env.parameter_heap, full_hash, txt_hash)?)?;
                } else {
                    return error(||"Provided value parameter must be of a supported type")
                }
            },

            ParamRef::Literal(index) => {
                if !p.primitive { return error(||"Literals must be of primitive type") }
                interpreter_stack.push(load_from_literal(env,*index,*p)?)?;
            },
            ParamRef::Witness(index) => {
                if !p.primitive { return error(||"Witnesses must be of primitive type") }
                interpreter_stack.push(load_from_witness(env,*index,*p)?)?;
            },
        };
    }

    ExecutionContext::interpret(&txt_desc.functions, &mut interpreter_stack, &mut frame_stack, &mut return_stack, &env.runtime_heap)?;

    //Now that we know it succeeds we can modify the store
    for del in deletes {
        delete_from_store(env,del)?
    }

    assert_eq!(interpreter_stack.len(), txt.returns.len(), "Transaction Return Information missmatched Stack");
    assert_eq!(interpreter_stack.len(), txt_desc.returns.len(), "Transaction Description Return Information missmatched Stack");


    for ((ret_entry, r), r_typ) in interpreter_stack.as_slice().iter().zip(txt_desc.returns.iter()).zip(txt.returns.iter()) {
        match *r {
            TxTReturn{ primitive, drop, typ, desc, .. } => {
                match r_typ {
                    RetType::Store => {
                        strats.created += 1;
                        if strats.created as u32 > env.available_entry_created  {return error(||"Bundle has not reserved enough entry creates")}
                        if primitive {return error(||"Can not store primitives") }
                        if !is_entry(typ) { return error(||"Stored return must be an entry") }
                        let id = unsafe {ret_entry.adt.1.get(0).expect("entry has to few fields").data.deref()}.try_into().expect("entry id has incorrect length");
                        let mut s = Serializer::new(CONFIG.max_structural_dept);
                        desc.serialize_value(*ret_entry, &mut s)?;
                        write_to_store(env, id, s.extract(), typ)?

                    },
                    RetType::Drop => if !drop { return error(||"Returns without drop capability must be stored") },
                    RetType::Log => {
                        if !drop { return error(||"Returns without drop capability must be stored") }
                        let mut s = Serializer::new(CONFIG.max_structural_dept);
                        desc.serialize_value(*ret_entry, &mut s)?;
                        let data = s.extract();
                        logger.log(data);
                    },
                }
            },
        }
    }
    Ok(strats)
}


//Helper to calc the key for a storage slot
fn entry_hash(typ:&[u8], data_hash:&Hash) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = HashingDomain::Entry.get_domain_hasher();
    //push the data into it
    context.update(&typ);
    context.update(data_hash);

    //calc the Hash
    context.finalize()
}

fn load_from_literal<'a,'b,'c, S:Store>(env:&ExecutionEnvironment<'a,'b, 'c,S>, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let ser_type = Serializer::serialize_fully(&param.typ,CONFIG.max_structural_dept)?;
    let mut context = Hasher::new();
    context.update(&ser_type);
    let control_hash =context.finalize();

    let entry_copy = env.literal_cache.borrow()[index as usize];
    Ok(match entry_copy {
        ParamCache::Unloaded => {
            let data = env.txt_bundle.literal[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.entry_cache.borrow_mut()[index as usize] = ParamCache::Loaded(entry, control_hash);
            entry
        },
        ParamCache::Loaded(entry, cache_control_hash) => {
            if control_hash != cache_control_hash { return error(||"A provided literal can not not be used multiple times with different types")}
            entry
        }
    })
}

fn load_from_witness<'a,'b,'c, S:Store>(env:&ExecutionEnvironment<'a,'b, 'c,S>, index:u16, param:TxTParam) -> Result<Entry<'b>> {
    let ser_type = Serializer::serialize_fully(&param.typ,CONFIG.max_structural_dept)?;
    let mut context = Hasher::new();
    context.update(&ser_type);
    let control_hash =context.finalize();

    let entry_copy = env.witness_cache.borrow()[index as usize];
    Ok(match entry_copy {
        ParamCache::Unloaded => {
            let data = env.txt_bundle.witness[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.entry_cache.borrow_mut()[index as usize] = ParamCache::Loaded(entry, control_hash);
            entry
        },
        ParamCache::Loaded(entry, cache_control_hash) => {
            if control_hash != cache_control_hash { return error(||"A provided literal can not not be used multiple times with different types")}
            entry
        }
    })
}


fn load_from_store<'a,'b,'c, S:Store>(env:&ExecutionEnvironment<'a,'b, 'c,S>, index:u16, hash:&Hash, param:TxTParam) -> Result<Entry<'b>> {
    if !is_entry(param.typ) { return error(|| "Value parameter must be an entry") }

    fn check_hash<'a,'b,'c, S:Store>(env:&ExecutionEnvironment<'a,'b, 'c,S>, typ:Ptr<RuntimeType>, key_hash:&Hash, value_hash:&Hash) -> Result<()>{
        let control_type = Serializer::serialize_fully(&typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type,&value_hash);

        let control_hash = env.store.get(StorageClass::EntryHash, key_hash,  |d|array_ref!(d,0,20).clone())?;
        if control_hash != expected_hash { return error(||"provided witness or stored value mismatched expected entry")}
        Ok(())
    }

    let entry_copy = env.entry_cache.borrow()[index as usize];
    Ok(match entry_copy {
        ParamCache::Unloaded => {
            let data = match env.txt_bundle.store_witness[index as usize] {
                None => {
                    if !STORE_MODE {return error(||"No witness was provided for store load and STORE MODE is off")}
                    env.store.get(StorageClass::EntryValue, hash, |d|d.to_vec())?
                },
                Some(data) => data.to_vec()
            };
            let mut data_hash = Hasher::new();
            data_hash.update(&data);
            let value_hash = data_hash.finalize();
            check_hash(env, param.typ, hash, &value_hash)?;

            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.entry_cache.borrow_mut()[index as usize] = ParamCache::Loaded(entry, value_hash);
            entry
        },
        ParamCache::Loaded(entry, value_hash) => {
            check_hash(env, param.typ, hash, &value_hash)?;
            entry
        }
    })
}

fn write_to_store<S:Store>(env:&ExecutionEnvironment<S>, id:Hash, data:Vec<u8>, provided_type:Ptr<RuntimeType>) -> Result<()> {
    let mut data_hash = Hasher::new();
    data_hash.update(&data);
    let value_hash = data_hash.finalize();
    let control_type = Serializer::serialize_fully(&provided_type,CONFIG.max_structural_dept)?;
    let expected_hash = entry_hash(&control_type, &value_hash);
    env.store.set(StorageClass::EntryHash, id, expected_hash.to_vec())?;

    if STORE_MODE {
        env.store.set(StorageClass::EntryValue, id,  data)?;
    }
    Ok(())
}

fn delete_from_store<S:Store>(env:&ExecutionEnvironment<S>, id:&Hash) -> Result<()> {
    if STORE_MODE {
        env.store.delete(StorageClass::EntryValue, id)?;
    }
    env.store.delete(StorageClass::EntryHash, id)
}

fn commit_store<S:Store>(env:&ExecutionEnvironment<S>) -> Result<()> {
    if STORE_MODE {
        env.store.commit(StorageClass::EntryValue)
    }
    Ok(env.store.commit(StorageClass::EntryHash))
}

fn rollback_store<S:Store>(env:&ExecutionEnvironment<S>) -> Result<()> {
    if STORE_MODE {
        env.store.rollback(StorageClass::EntryValue)
    }
    Ok(env.store.rollback(StorageClass::EntryHash))
}

pub fn create_ctx<'a,'h>(alloc:&'a VirtualHeapArena<'h>, full_hash:&Hash, txt_hash:&Hash) -> Result<Entry<'a>> {
    //Todo: construct over schema to be compiler agnostic
    Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
        Entry {data: alloc.copy_alloc_slice(txt_hash)?},
        Entry {data: alloc.copy_alloc_slice(full_hash)?},
        Entry {u64: 0},
        Entry {u64: 0}
    ])?)})
}