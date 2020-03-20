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
use model::{Transaction, ParamRef, TypedData, RetType, TransactionBundle, ParamMode};
use sanskrit_common::model::{Hash, SlicePtr, Ptr};
//use ed25519_dalek::*;
//use sha2::{Sha512};
use sanskrit_common::arena::*;
use sanskrit_interpreter::interpreter::{Frame, ExecutionContext};
use sanskrit_common::hashing::HashingDomain;
use sanskrit_interpreter::model::{Entry, Adt, TransactionDescriptor, TxTParam, TxTReturn, ValueSchema, RuntimeType};
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use system::{is_entry, is_context};
use core::convert::TryInto;
use core::ops::Deref;

pub mod model;
mod system;

//Todo: The gas & entry & volume Accounting for storage is clunky
//      We need a better solution

//Todo: We need a store Entry Limit to prevent attacks when storing to much
//      This is needed as we account after the fact
//      With that we can check that we have enough for the worst case

pub const STORAGE_COST: DataProcessingCost = DataProcessingCost {
    //Todo: Just a guesses probably a bad ones
    base_cost: 2000,
    cost_multiplier: 50,
    cost_divider: 1
};


pub const ENCODING_COST: DataProcessingCost = DataProcessingCost {
    //Todo: Just a guesses probably a bad ones
    base_cost: 10,
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
    encoding_cost: ENCODING_COST,
    store_cost: STORAGE_COST,
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
    pub load_cost: usize,
    pub encoding_cost: DataProcessingCost,
    pub store_cost: DataProcessingCost,
}

pub struct DataProcessingCost {
    pub base_cost:u64,
    pub cost_multiplier:u64,
    pub cost_divider:u64,
}


//A struct holding context information of the current transaction
pub struct ExecutionEnvironment<'a, 'b, S:Store> {
    store:&'a S,
    max_desc_alloc:VirtualHeapArena<'b>,
    txt_bundle:TransactionBundle<'b>,
    structural_arena:HeapArena<'b>,
    parameter_heap:VirtualHeapArena<'b>,
    runtime_heap:VirtualHeapArena<'b>,
    required_gas_prepay:u64,
    required_entry_prepay:u64,
    required_volume_prepay:u64,
    //todo: we need to ensure that it is always parsed with the same desc even if cached
    param_cache:Vec<Entry<'b>>,
    literal_cache:Vec<Entry<'b>>,
    witness_cache:Vec<Entry<'b>>,
    available_gas:u64,
    gas_refund:u64,
    entries_refund:u64,
    volume_refund:u64,
}

pub trait Logger {
    fn log<'a>(&mut self, data:Vec<u8>);
}

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
    let mut required_gas_prepay = 0;
    let mut required_entry_prepay = 0;
    let mut required_volume_prepay = 0;

    for txt_section in txt_bundle.sections.iter() {
        required_gas_prepay += txt_section.gas_limit as u64;
        required_entry_prepay += txt_section.extra_entries_limit as u64;
        required_volume_prepay += txt_section.storage_volume_limit as u64;
    }

    let witness_size = txt_bundle.witness.iter().map(|w|w.len() + 2).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
    let store_witness_size = txt_bundle.store_witness.iter().map(|w|w.len() + 2).sum::<usize>() + 2;  //2 is num wittness / Num Bytes
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

    let mut storage_alloc = heap.new_virtual_arena(txt_bundle.storage_cache as usize);
    let mut parameter_heap = heap.new_virtual_arena(txt_bundle.param_heap_limit as usize);
    let mut runtime_heap = heap.new_virtual_arena(txt_bundle.runtime_heap_limit as usize);

    let mut exec_env = ExecutionEnvironment {
        store,
        max_desc_alloc: storage_alloc,
        txt_bundle,
        structural_arena,
        parameter_heap,
        runtime_heap,
        required_gas_prepay,
        required_entry_prepay,
        required_volume_prepay,
        param_cache: Vec::new(),
        literal_cache: Vec::new(),
        witness_cache: Vec::new(),
        available_gas:0,        //will be set for each section
        gas_refund:0,           //will be increased for each section
        entries_refund:0,       //will be increased for each section
        volume_refund:0,        //will be increased for each section
    };

    if exec_env.txt_bundle.store_witness.len() != exec_env.txt_bundle.stored.len() {return error(||"Not enough witnesses for stored values")}

    //note we account the parsing on the source bytes not the target bytes
    for p in exec_env.txt_bundle.store_witness.iter() {
        //accounts for the parsing & loading of the hash
        required_gas_prepay +=  (CONFIG.load_cost  as u64) + ((p.len() as u64)*CONFIG.encoding_cost.cost_multiplier)/CONFIG.encoding_cost.cost_divider;
    }

    for txt_section in exec_env.txt_bundle.sections.iter() {
        exec_env.available_gas = txt_section.gas_limit;
        for txt in txt_section.txts.iter() {
            match execute_transaction( &exec_env, txt, &full_bundle_hash,  &bundle_hash, logger ) {
                Ok(gas) => {
                    //charge for the used gas
                    exec_env.available_gas -= gas as u64
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



        //Before commit check that limits hold
        let ChangeReport { entries_difference, bytes_difference} = store.report(StorageClass::EntryValue);
        if entries_difference > txt_section.extra_entries_limit as isize{
            return error(||"Stored entries exceeds limit")
        }

        if bytes_difference > txt_section.storage_volume_limit as isize{
            return error(||"Stored volume exceeds limit")
        }

        //Todo: Check that local Store has no consumables (linears + affines)

        //Todo: Account for the actual storing
        //      We need a more detailed report for this
        //      Do the same precharge and recharge we do on a per transaction level

        //commit
        commit_store(&exec_env)?;

        //calculate and add the refund
        exec_env.entries_refund += (txt_section.extra_entries_limit as isize - entries_difference) as u64;
        exec_env.volume_refund += (txt_section.storage_volume_limit as isize - bytes_difference) as u64;
    }

    Ok(())
}

fn execute_transaction<S:Store, L:Logger>(env:&ExecutionEnvironment<S>,txt:&Transaction,full_hash:&Hash,txt_hash:&Hash,logger:&mut L) -> Result<u32>{

    //Prepare all the Memory
    if env.txt_bundle.descriptors.len() <= txt.txt_desc as usize { return error(||"Descriptor index out of range")  }

    //todo: shall we do at the start
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

    for (p,p_typ) in txt_desc.params.iter().zip(txt.params.iter()) {
        match p_typ {
            ParamRef::Load(ParamMode::Consume,index) => {
                if !p.consumes && !p.drop { return error(||"A owned store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                //We delete at end so others can copy and in case it produces an error it must still be their
                deletes.push(hash);
                let data = load_from_store(env,hash,p.typ)?;
                let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
                interpreter_stack.push(p.desc.parse_value(&mut parser, &env.parameter_heap)?)?;
            }
            ParamRef::Load(ParamMode::Copy, index) => {
                if !p.copy { return error(||"A Copied store value must allow copy") }
                if !p.consumes && !p.drop { return error(||"A Copied store value must be consumed or dropped") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                let data = load_from_store(env,hash,p.typ)?;
                let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
                interpreter_stack.push(p.desc.parse_value(&mut parser, &env.parameter_heap)?)?;
            },
            ParamRef::Load(ParamMode::Borrow, index) => {
                if p.consumes { return error(||"A Borrowed store value can not be consumed") }
                if p.primitive { return error(||"Primitives can not be loaded from store") }
                if env.txt_bundle.stored.len() <= *index as usize { return error(||"Value index out of range")  }
                let hash = array_ref!(&env.txt_bundle.stored[*index as usize],0,20);
                if lock_set.contains(hash) { return error(||"An entry can only be fetched once"); }
                lock_set.insert(hash.clone());
                let data = load_from_store(env,hash,p.typ)?;
                let mut parser = Parser::new(&data, CONFIG.max_structural_dept);
                interpreter_stack.push(p.desc.parse_value(&mut parser, &env.parameter_heap)?)?;
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
                if env.txt_bundle.literal.len() <= *index as usize { return error(||"Literal index out of range")  }
                let mut parser = Parser::new(&env.txt_bundle.literal[*index as usize ], CONFIG.max_structural_dept);
                interpreter_stack.push(p.desc.parse_value(&mut parser, &env.parameter_heap)?)?;
            },
            ParamRef::Witness(index) => {
                if !p.primitive { return error(||"Witnesses must be of primitive type") }
                if env.txt_bundle.literal.len() <= *index as usize { return error(||"Witness index out of range")  }
                let mut parser = Parser::new(&env.txt_bundle.witness[*index as usize ], CONFIG.max_structural_dept);
                interpreter_stack.push(p.desc.parse_value(&mut parser, &env.parameter_heap)?)?;
            },

            ParamRef::Fetch(ParamMode::Consume,index) => {
                //todo:
            },

            ParamRef::Fetch(ParamMode::Copy,index) => {
                //todo:
            },

            ParamRef::Fetch(ParamMode::Borrow,index) => {
                //todo:
            },
        };
    }

    //todo: select interpreter based on compiler
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
                        if primitive {return error(||"Can not store primitives") }
                        if !is_entry(typ) { return error(||"Stored return must be an entry") }
                        let id = unsafe {ret_entry.adt.1.get(0).expect("entry has to few fields").data.deref()}.try_into().expect("entry id has incorrect length");
                        let mut s = Serializer::new(CONFIG.max_structural_dept);
                        desc.serialize_value(*ret_entry, &mut s)?;
                        write_to_store(env, id, s.extract(), typ)?

                    },
                    RetType::Put(index) => {
                        //todo: implement
                        // 1: check that nothing is stored under index
                        // 2: use schema to serialize
                        // 3: construct TypedData
                        // 4: store at index
                        // 5: if !drop increase consumable count (this must be 0 at section borders)
                        // Note: Account for storing in gas
                    }

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
    Ok(txt_desc.gas_cost)
}


fn load_from_store<S:Store>(env:&ExecutionEnvironment<S>, hash:&Hash, expected_type:Ptr<RuntimeType>) -> Result<Vec<u8>> {
    //todo: this should be eliminated
    let data:TypedData = env.store.parsed_get(StorageClass::EntryValue, hash, CONFIG.max_structural_dept, &env.max_desc_alloc)?;
    if !is_entry(expected_type) { return error(|| "Value parameter must be an entry") }
    //todo: is this expensive?? or dominated by loading??? -- Probably the Later
    if expected_type != data.typ { return error(|| "Data in store has wrong type") }
    return Ok(data.value)
}

fn write_to_store<S:Store>(env:&ExecutionEnvironment<S>, id:Hash, data:Vec<u8>, provided_type:Ptr<RuntimeType>) -> Result<()> {
    let data = TypedData {
        typ:provided_type,
        value: data
    };
    env.store.serialized_set(StorageClass::EntryValue, id, CONFIG.max_structural_dept, &data)
}

fn delete_from_store<S:Store>(env:&ExecutionEnvironment<S>, id:&Hash) -> Result<()> {
    env.store.delete(StorageClass::EntryValue, id)
}

fn commit_store<S:Store>(env:&ExecutionEnvironment<S>) -> Result<()> {
    Ok(env.store.commit(StorageClass::EntryValue))
}

fn rollback_store<S:Store>(env:&ExecutionEnvironment<S>) -> Result<()> {
    Ok(env.store.rollback(StorageClass::EntryValue))
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