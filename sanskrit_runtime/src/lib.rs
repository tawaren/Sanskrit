#![no_std]
#![feature(nll)]

#[macro_use]
extern crate arrayref;
extern crate byteorder;
extern crate num_traits;
//extern crate ed25519_dalek;
//extern crate sha2;
extern crate sanskrit_core;
extern crate sanskrit_deploy;
extern crate sanskrit_compile;
extern crate sanskrit_interpreter;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;
extern crate alloc;


use sanskrit_common::store::{Store, StorageClass};
use sanskrit_common::errors::*;
use sanskrit_common::encoding::{Parser, Serializer, ParserAllocator};
use model::{Transaction, ParamRef, RetType, TransactionBundle, ParamMode, DeployType, BundleSection};
use sanskrit_common::model::{Hash, Ptr, hash_from_slice};
//use ed25519_dalek::*;
//use sha2::{Sha512};
use sanskrit_common::arena::*;
use sanskrit_interpreter::interpreter::{Frame, ExecutionContext};
use sanskrit_common::hashing::{HashingDomain, Hasher};
use sanskrit_interpreter::model::{Entry, Adt, TransactionDescriptor, TxTParam, TxTReturn, RuntimeType};
use alloc::vec::Vec;

use alloc::collections::BTreeSet;
use system::System;
use core::convert::TryInto;
use core::ops::Deref;
use core::cell::{RefCell, Cell};
use sanskrit_deploy::{deploy_module, deploy_function};
use sanskrit_core::accounting::Accounting;
use sanskrit_compile::compile_function;
use sanskrit_compile::limiter::Limiter;
use sanskrit_interpreter::externals::CompilationExternals;

pub mod model;
pub mod system;

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

//Todo: Enforce these, check they are not reached
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
    block_inclusion_window: 100,
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
    pub block_inclusion_window:u64,
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

pub trait Tracker {
    fn section_start(&mut self, section:&BundleSection);
     fn transaction_start(&mut self, transaction:&Transaction);
       fn parameter_load(&mut self, p_ref:&ParamRef, p_desc:&TxTParam, value:&Entry);
       fn return_value(&mut self, r_typ:&RetType, r_desc:&TxTReturn, value:&Entry);
     fn transaction_finish(&mut self, transaction:&Transaction, success:bool);
    fn section_finish(&mut self, section:&BundleSection, success:bool);
}

//todo: should this be in Local Server or another Wrapper that has the Store???
//Helper for outside transaction builder
pub fn read_elem<S:Store>(hash:&Hash, store:&S) -> Result<Vec<u8>>{
    store.get(StorageClass::EntryValue, hash, |d|d.to_vec())
}

//todo: should this be in Local Server or another Wrapper that has the Store???
//Helper for outside transaction builder
pub fn read_transaction_desc<'a, S:Store, A:ParserAllocator>(target:&Hash, store:&S, heap:&'a A) -> Result<TransactionDescriptor<'a>>{
    store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, heap)
}

//TODO: Make cmplete rework of the fast stuff and see if with the witnesses we can get it in order
//      In storage / provider mode do not execute payment txts (for now)
//

//Todo: We need evaluation Mode
//      Takes Bundle but with limits & ev: witnesses set to 0/null
//      Then it evaluates them and returns the filled bundle data

//Executes a transaction
pub fn execute<S:Store, L: Tracker, E:CompilationExternals, SYS:System>(store:&S, sys:&SYS, txt_bundle_data:&[u8], block_no:u64, heap:&Heap, tracker:&mut L, system_mode_on:bool) -> Result<()> {
    //Check that it is inside limit
    if txt_bundle_data.len() > CONFIG.max_transaction_size { return error(||"Transaction Bundle to big")}
    //Static allocations (could be done once)
    // A buffer to parse the transaction and load values from store
    let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_transaction_memory);
    //Parse the transaction
    let txt_bundle:TransactionBundle = Parser::parse_fully(txt_bundle_data, CONFIG.max_structural_dept, &txt_bundle_alloc)?;
    //check that it is in window
    if block_no < txt_bundle.core.earliest_block || block_no >= txt_bundle.core.earliest_block + CONFIG.block_inclusion_window {
        return error(||"Transaction not allowed in current block")
    }

    //Calculate the payment information
    //Starts with the parsing costs which are already done mostly but this is inevitable (a miner could have a size limit)
    //This includes encoding the parameter witnesses
    //todo: only thin missing is the cost for loading & parsing the Transactions themself
    //      but these are predictable and could be set per section and then checked
    let mut required_gas_prepay = (CONFIG.parsing_cost.cost_multiplier * (txt_bundle_data.len() as u64))/ CONFIG.parsing_cost.cost_divider;
    let mut required_entry_storage_prepay = 0;


    let mut change_in_stored_entries = 0;
    for txt_section in txt_bundle.core.sections.iter() {
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

    //todo; is this right here or do we need this later??
    if txt_bundle.byte_size.unwrap() - txt_bundle.core.byte_size.unwrap() - 4 /*the -4 is the 2 sizes*/ > txt_bundle.core.witness_bytes_limit as usize {
        return error(||"Transaction Bundle witness size exceeds wittness limit")
    }

    let full_bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data);
    let bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data[..txt_bundle.core.byte_size.unwrap()]);

    //Create Allocator
    //create heaps: based on bundle input
    let mut structural_arena = heap.new_arena(
        Heap::elems::<Entry>(txt_bundle.core.stack_elem_limit as usize)
            + Heap::elems::<Frame>(txt_bundle.core.stack_frame_limit as usize)
            + Heap::elems::<Entry>(CONFIG.return_stack)
    );

    let mut max_desc_alloc = heap.new_virtual_arena(txt_bundle.core.transaction_storage_heap as usize);
    let mut parameter_heap = heap.new_virtual_arena(txt_bundle.core.param_heap_limit as usize);
    let mut runtime_heap = heap.new_virtual_arena(txt_bundle.core.runtime_heap_limit as usize);

    let entry_cache = RefCell::new(alloc::vec::from_elem(ParamCache::Unloaded, txt_bundle.store_witness.len()));
    let literal_cache = RefCell::new(alloc::vec::from_elem(ParamCache::Unloaded, txt_bundle.core.literal.len()));
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

    if exec_env.txt_bundle.store_witness.len() != exec_env.txt_bundle.core.stored.len() { return error(||"Not enough witnesses for stored values") }

    for txt_section in exec_env.txt_bundle.core.sections.iter() {
        tracker.section_start(txt_section);
        exec_env.available_gas = txt_section.gas_limit;
        exec_env.available_entry_loaded = txt_section.entries_loaded;
        exec_env.available_entry_created = txt_section.entries_created;
        exec_env.available_entry_deleted = txt_section.entries_deleted;

        for txt in txt_section.txts.iter() {
            tracker.transaction_start(txt);
            match execute_transaction(&exec_env, sys, txt, &full_bundle_hash, &bundle_hash, block_no, tracker) {
                Ok(stats) => {
                    //charge for the used gas
                    exec_env.available_gas -= stats.gas as u64;
                    exec_env.available_entry_loaded -= stats.loaded as u32;
                    exec_env.available_entry_created -= stats.created as u32;
                    exec_env.available_entry_deleted -= stats.deleted as u32;
                },
                Err(err) => {
                    rollback_store(&exec_env)?;
                    tracker.transaction_finish(txt, false);
                    tracker.section_finish(txt_section, false);
                    return Err(err);
                }
            };

            //release all the memory so it does not leak into the next transaction
            exec_env.structural_arena = exec_env.structural_arena.reuse();
            exec_env.max_desc_alloc = exec_env.max_desc_alloc.reuse();
            exec_env.runtime_heap = exec_env.runtime_heap.reuse();
            tracker.transaction_finish(txt, true);
        }

        //commit
        commit_store(&exec_env)?;
        tracker.section_finish(txt_section, true);

    }

    match exec_env.txt_bundle.core.deploy {
        None =>  Ok(()),
        Some(depl) => {
            //todo: check these are not more than a static upper bound?

            let accounting = Accounting {
                load_byte_budget: Cell::new(depl.max_load_bytes as usize),
                store_byte_budget: Cell::new(depl.max_store_bytes as usize),
                process_byte_budget: Cell::new(depl.max_process_bytes as usize),
                stack_elem_budget: Cell::new(depl.max_stack_elems as usize),
                max_nesting: Cell::new(0),
                nesting_limit: depl.max_block_nesting as usize,
                input_limit: depl.data.len()
            };
            match depl.typ {
                DeployType::Module => {
                    //todo: I do not like the to_vec here (as we have it in memory twice now)
                    //but without having seperate Transaction type it is hard not to do this
                    //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
                    deploy_module(exec_env.store, &accounting, depl.data.to_vec(), system_mode_on, true)?;
                    exec_env.store.commit(StorageClass::Module);
                },
                DeployType::Transaction => {
                    //todo: I do not like the to_vec here (as we have it in memory twice now)
                    //but without having seperate Transaction type it is hard not to do this
                    //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
                    let target = deploy_function(exec_env.store, &accounting, depl.data.to_vec(), true)?;
                    let limiter = Limiter {
                        max_functions: depl.max_contained_functions as usize,
                        max_nesting: depl.max_compile_block_nesting as usize,
                        max_used_nesting: Cell::new(0),
                        produced_functions: Cell::new(0)
                    };
                    compile_function::<S,E>(exec_env.store, &accounting,&limiter, target, true)?;
                    exec_env.store.commit(StorageClass::Transaction);
                    exec_env.store.commit(StorageClass::Descriptor);
                },
            }
            Ok(())
        },
    }
}



fn execute_transaction<S:Store, L: Tracker, SYS:System>(env:&ExecutionEnvironment<S>, sys:&SYS, txt:&Transaction, full_hash:&Hash, txt_hash:&Hash, block_no:u64, tracker:&mut L) -> Result<TxTStats>{

    //Prepare all the Memory
    if env.txt_bundle.core.descriptors.len() <= txt.txt_desc as usize { return error(||"Descriptor index out of range")  }
    let target = &env.txt_bundle.core.descriptors[txt.txt_desc as usize];
    //todo: pre calculate this: how to gas charge this
    let txt_desc:TransactionDescriptor = env.store.parsed_get(StorageClass::Descriptor, target, CONFIG.max_structural_dept, &env.max_desc_alloc)?;

    if txt_desc.gas_cost as u64 > env.available_gas {return error(||"Bundle has not reserved enough gas")}
    if txt_desc.max_frames > env.txt_bundle.core.stack_frame_limit {return error(||"Bundle has not reserved enough frame space")}
    if txt_desc.max_stack > env.txt_bundle.core.stack_elem_limit {return error(||"Bundle has not reserved enough stack space")}
    if txt_desc.max_mem > env.txt_bundle.core.runtime_heap_limit {return error(||"Bundle has not reserved enough heap space")}

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
                if env.txt_bundle.core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &env.txt_bundle.core.stored[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                //We delete at end so others can copy and in case it produces an error it must still be their
                deletes.push(hash);
                let data = load_from_store(env, sys, *index, hash,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            }
            ParamRef::Load(ParamMode::Copy, index) => {
                strats.loaded+=1;
                if strats.loaded as u32 > env.available_entry_loaded  {return error(||"Bundle has not reserved enough entry loads")}
                if !p.copy { return error(||"A Copied store value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A Copied store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &env.txt_bundle.core.stored[*index as usize];
                let data = load_from_store(env, sys, *index, hash,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
            ParamRef::Load(ParamMode::Borrow, index) => {
                strats.loaded+=1;
                if strats.loaded as u32 > env.available_entry_loaded  {return error(||"Bundle has not reserved enough entry loads")}
                if p.consumes { return error(||"A Borrowed store value can not be consumed") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.core.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = &env.txt_bundle.core.stored[*index as usize];
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                let data = load_from_store(env, sys, *index, hash,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },

            ParamRef::Provided => {
                if sys.is_context(p.typ) {
                    let data = create_ctx(&env.parameter_heap, full_hash, txt_hash, block_no)?;
                    tracker.parameter_load(p_typ, p, &data);
                    interpreter_stack.push(data)?;
                } else {
                    return error(||"Provided value parameter must be of a supported type")
                }
            },

            ParamRef::Literal(index) => {
                if !p.primitive { return error(||"Literals must be of primitive type") }
                let data = load_from_literal(env,*index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
            },
            ParamRef::Witness(index) => {
                if !p.primitive { return error(||"Witnesses must be of primitive type") }
                let data = load_from_witness(env,*index,*p)?;
                tracker.parameter_load(p_typ, p, &data);
                interpreter_stack.push(data)?;
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
                        if !sys.is_entry(typ) { return error(||"Stored return must be an entry") }
                        let id = unsafe {ret_entry.adt.1.get(0).expect("entry has to few fields").data.deref()}.try_into().expect("entry id has incorrect length");
                        let mut s = Serializer::new(CONFIG.max_structural_dept);
                        tracker.return_value(r_typ, r, ret_entry);
                        desc.serialize_value(*ret_entry, &mut s)?;
                        write_to_store(env, id, s.extract(), typ)?

                    },
                    RetType::Drop => {
                        if !drop { return error(||"Returns without drop capability must be stored") }
                        tracker.return_value(r_typ, r, ret_entry);
                    },
                    RetType::Log => {
                        if !drop { return error(||"Returns without drop capability must be stored") }
                        let mut s = Serializer::new(CONFIG.max_structural_dept);
                        tracker.return_value(r_typ, r, ret_entry);
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
            let data = env.txt_bundle.core.literal[index as usize];
            let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
            let entry = param.desc.parse_value(&mut parser, env.parameter_heap)?;
            env.literal_cache.borrow_mut()[index as usize] = ParamCache::Loaded(entry, control_hash);
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
            env.witness_cache.borrow_mut()[index as usize] = ParamCache::Loaded(entry, control_hash);
            entry
        },
        ParamCache::Loaded(entry, cache_control_hash) => {
            if control_hash != cache_control_hash { return error(||"A provided literal can not not be used multiple times with different types")}
            entry
        }
    })
}



fn load_from_store<'a,'b,'c, S:Store, SYS:System>(env:&ExecutionEnvironment<'a,'b, 'c,S>, sys:&SYS, index:u16, hash:&Hash, param:TxTParam) -> Result<Entry<'b>> {
    if !sys.is_entry(param.typ) { return error(|| "Value parameter must be an entry") }

    fn check_hash<'a,'b,'c, S:Store>(env:&ExecutionEnvironment<'a,'b, 'c,S>, typ:Ptr<RuntimeType>, key_hash:&Hash, value_hash:&Hash) -> Result<()>{
        let control_type = Serializer::serialize_fully(&typ,CONFIG.max_structural_dept)?;
        let expected_hash = entry_hash(&control_type,&value_hash);

        let control_hash = env.store.get(StorageClass::EntryHash, key_hash,  |d|hash_from_slice(d))?;
        if control_hash != expected_hash { return error(||"provided witness or stored value mismatched expected entry")}
        Ok(())
    }

    let entry_copy = env.entry_cache.borrow()[index as usize];
    Ok(match entry_copy {
        ParamCache::Unloaded => {
            let data = match env.txt_bundle.store_witness[index as usize] {
                None => {
                    if !STORE_MODE {return error(||"No witness was provided for store load and STORE MODE is off")}
                    read_elem(hash, env.store)?
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

pub fn create_ctx<'a,'h>(alloc:&'a VirtualHeapArena<'h>, full_hash:&Hash, txt_hash:&Hash, block_no:u64) -> Result<Entry<'a>> {
    //Todo: construct over schema to be compiler agnostic
    Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
        Entry {data: alloc.copy_alloc_slice(txt_hash)?},
        Entry {data: alloc.copy_alloc_slice(full_hash)?},
        Entry {u64: block_no},
        Entry {u64: 0}
    ])?)})
}