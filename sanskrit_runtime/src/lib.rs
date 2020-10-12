#![no_std]
#![feature(nll)]

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
use sanskrit_common::encoding::{VirtualSize, Parser, ParserAllocator};
use model::{Transaction, ParamRef, RetType, BundleSection, TransactionBundle, DeployTransaction, DeployType};
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::{Entry, TxTParam, TxTReturn, TransactionDescriptor};

use system::System;
use sanskrit_interpreter::externals::CompilationExternals;
use verify::{verify_repeated, verify_once};
use compute::execute_once;
use sanskrit_common::model::Hash;
use sanskrit_interpreter::interpreter::Frame;
use sanskrit_common::hashing::HashingDomain;
use direct_stored::{StatefulEntryStoreAccounter, StatefulEntryStoreExecutor};
use sanskrit_core::accounting::Accounting;
use core::cell::Cell;
use sanskrit_deploy::{deploy_module, deploy_function};
use sanskrit_compile::limiter::Limiter;
use sanskrit_compile::compile_function;

pub mod model;
pub mod system;
pub mod verify;
pub mod direct_stored;
pub mod compute;

pub struct DataProcessingCost {
    cost_constant:u64,
    cost_multiplier:u64,
    cost_divider:u64,
}

impl DataProcessingCost {
    pub fn compute(&self, amount:u64) -> u64 {
        self.cost_constant + (amount * self.cost_multiplier)/self.cost_divider
    }
}

//TODO: Measure, these are guessed
pub const STORE_LOAD_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  200 - Hash::SIZE as u64,
    cost_multiplier: 1,
    cost_divider: 1
};

pub const STORE_WRITE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  2000 - Hash::SIZE as u64,
    cost_multiplier: 10,
    cost_divider: 1
};

pub const STORE_LOAD_AND_ENCODE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  200 - Hash::SIZE as u64,
    cost_multiplier: 6,
    cost_divider: 5
};

pub const STORE_WRITE_AND_ENCODE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  2000 - Hash::SIZE as u64,
    cost_multiplier: 51,
    cost_divider: 5
};

pub const ENCODING_COST: DataProcessingCost = DataProcessingCost {
    cost_constant: 0,
    cost_multiplier: 1,
    cost_divider: 5
};

pub struct Configuration {
    pub max_stack_depth:usize,
    pub max_frame_depth:usize,
    pub max_heap_size:usize,
    pub max_transaction_size: usize,
    pub max_structural_dept: usize,
    pub max_transaction_memory: usize,
    pub return_stack: usize,
    pub entry_load_cost: DataProcessingCost,
    pub entry_store_cost: DataProcessingCost,
    pub txt_desc_load_cost:DataProcessingCost,
    pub parsing_cost: DataProcessingCost,
    pub block_inclusion_window:u64,
}

pub const CONFIG: Configuration = Configuration {
    max_stack_depth:2048,
    max_frame_depth:512,
    max_heap_size:512 * 1024,
    max_transaction_size: 128 * 1024,
    max_structural_dept:64,
    max_transaction_memory:512 * 1024,
    return_stack: 256,
    entry_store_cost: STORE_WRITE_AND_ENCODE_COST,
    entry_load_cost: STORE_LOAD_AND_ENCODE_COST,
    txt_desc_load_cost: STORE_LOAD_COST,
    parsing_cost: ENCODING_COST,
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


pub trait Tracker {
    fn section_start(&mut self, section:&BundleSection);
    fn transaction_start(&mut self, transaction:&Transaction);
    fn parameter_load(&mut self, p_ref:&ParamRef, p_desc:&TxTParam, value:&Entry);
    fn return_value(&mut self, r_typ:&RetType, r_desc:&TxTReturn, value:&Entry);
    fn transaction_finish(&mut self, transaction:&Transaction, success:bool);
    fn section_finish(&mut self, section:&BundleSection, success:bool);
}

pub fn read_transaction_desc<'d, S:Store, A:ParserAllocator>(target:&Hash, store:&S, heap: &'d A) -> Result<TransactionDescriptor<'d>> {
    direct_stored::read_transaction_desc(target, store, heap)
}


//Executes a transaction
pub fn execute<S:Store, L: Tracker, E:CompilationExternals, SYS:System>(store:&S, sys:&SYS, txt_bundle_data:&[u8], block_no:u64, heap:&Heap, tracker:&mut L) -> Result<()> {
    //Check that it is inside limit
    if txt_bundle_data.len() > CONFIG.max_transaction_size { return error(||"Transaction Bundle to big")}
    //Static allocations (could be done once)
    // A buffer to parse the transaction and load values from store
    let txt_bundle_alloc = heap.new_virtual_arena(CONFIG.max_transaction_memory);
    //Parse the transaction
    let txt_bundle:TransactionBundle = Parser::parse_fully(txt_bundle_data, CONFIG.max_structural_dept, &txt_bundle_alloc)?;
    let full_bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data);
    let bundle_hash = HashingDomain::Bundle.hash(&txt_bundle_data[..txt_bundle.core.byte_size.unwrap()]);

    //create the context
    verify_repeated(&txt_bundle, block_no)?;
    verify_once(&StatefulEntryStoreAccounter::new(store,&txt_bundle), sys, heap)?;
    execute_once(&StatefulEntryStoreExecutor::new(store,&txt_bundle,bundle_hash, full_bundle_hash), block_no, heap, tracker)
}

pub fn deploy<S:Store, E:CompilationExternals, >(store:&S, deploy_data:&[u8], heap:&Heap, system_mode_on:bool) -> Result<()> {
    //Check that it is inside limit
    if deploy_data.len() > CONFIG.max_transaction_size { return error(||"Transaction Bundle to big")}
    //Static allocations (could be done once)
    // A buffer to parse the transaction and load values from store
    let deploy_txt_alloc = heap.new_virtual_arena(CONFIG.max_transaction_memory);
    //Parse the transaction
    let deploy_txt:DeployTransaction = Parser::parse_fully(deploy_data, CONFIG.max_structural_dept, &deploy_txt_alloc)?;

    let accounting = Accounting {
        load_byte_budget: Cell::new(deploy_txt.max_load_bytes as usize),
        store_byte_budget: Cell::new(deploy_txt.max_store_bytes as usize),
        process_byte_budget: Cell::new(deploy_txt.max_process_bytes as usize),
        stack_elem_budget: Cell::new(deploy_txt.max_stack_elems as usize),
        max_nesting: Cell::new(0),
        nesting_limit: deploy_txt.max_block_nesting as usize,
        input_limit: deploy_txt.data.len()
    };
    match deploy_txt.typ {
        DeployType::Module => {
            //todo: I do not like the to_vec here (as we have it in memory twice now)
            //but without having seperate Transaction type it is hard not to do this
            //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
            deploy_module(store, &accounting, deploy_txt.data.to_vec(), system_mode_on, true)?;
            store.commit(StorageClass::Module);
        },
        DeployType::Transaction => {
            //todo: I do not like the to_vec here (as we have it in memory twice now)
            //but without having seperate Transaction type it is hard not to do this
            //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
            let target = deploy_function(store, &accounting, deploy_txt.data.to_vec(), true)?;
            let limiter = Limiter {
                max_functions: deploy_txt.max_contained_functions as usize,
                max_nesting: deploy_txt.max_compile_block_nesting as usize,
                max_used_nesting: Cell::new(0),
                produced_functions: Cell::new(0)
            };
            compile_function::<S,E>(store, &accounting,&limiter, target, true)?;
            store.commit(StorageClass::Transaction);
            store.commit(StorageClass::Descriptor);
        }
    }
    Ok(())
}