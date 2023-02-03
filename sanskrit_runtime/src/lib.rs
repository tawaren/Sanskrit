#![no_std]
#![feature(nll)]

extern crate arrayref;
extern crate byteorder;
extern crate num_traits;
//extern crate ed25519_dalek;
//extern crate sha2;
extern crate sanskrit_core;
#[cfg(feature = "deployer")]
extern crate sanskrit_deploy;
#[cfg(feature = "deployer")]
extern crate sanskrit_compile;

extern crate sanskrit_interpreter;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;
extern crate alloc;

#[cfg(feature = "deployer")]
use sanskrit_common::store::StorageClass;
use sanskrit_common::store::Store;
use sanskrit_common::errors::*;
#[cfg(feature = "deployer")]
use sanskrit_common::encoding::Parser;
use sanskrit_common::encoding::ParserAllocator;
use model::{Transaction, ParamRef, RetType, BundleSection};
#[cfg(feature = "deployer")]
use model::{DeployTransaction, DeployType};
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::{Entry, TxTParam, TxTReturn, TransactionDescriptor};

use system::SystemContext;
use verify::{verify_repeated, verify_once, TransactionVerificationContext};
use compute::{execute_once, TransactionExecutionContext};
use sanskrit_common::model::{Hash, SlicePtr};
use sanskrit_interpreter::interpreter::Frame;
#[cfg(feature = "deployer")]
use sanskrit_deploy::{deploy_module, deploy_function};
#[cfg(feature = "deployer")]
use sanskrit_compile::compile_function;
#[cfg(feature = "deployer")]
use sanskrit_compile::externals::CompilationExternals;

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
//Try to get approximately: 1000000 = 1ms / 1000gas == 1us / 1gas == 1ns (Use My Lenovo as referenz)
// We need to remeasure the primitives
pub const STORE_LOAD_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  20000,
    cost_multiplier: 10,
    cost_divider: 1
};

pub const STORE_WRITE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  20000,
    cost_multiplier: 50,
    cost_divider: 1
};

pub const STORE_LOAD_AND_ENCODE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  2000,
    cost_multiplier: 60,
    cost_divider: 5
};

pub const STORE_WRITE_AND_ENCODE_COST: DataProcessingCost = DataProcessingCost {
    cost_constant:  20000,
    cost_multiplier: 52,
    cost_divider: 1
};

pub const ENCODING_COST: DataProcessingCost = DataProcessingCost {
    cost_constant: 0,
    cost_multiplier: 2,
    cost_divider: 1
};

pub const COPYING_COST: DataProcessingCost = DataProcessingCost {
    cost_constant: 0,
    cost_multiplier: 2,
    cost_divider: 1
};

pub struct Configuration {
    pub max_stack_depth:usize,
    pub max_frame_depth:usize,
    pub max_heap_size:usize,
    pub max_bundle_size: usize,
    pub max_txt_alloc: usize,
    pub max_structural_dept: usize,
    pub return_stack: usize,
    pub bundle_base_cost:u64,
    pub entry_load_cost: DataProcessingCost,
    pub entry_store_cost: DataProcessingCost,
    pub txt_desc_load_cost:DataProcessingCost,
    pub parsing_cost: DataProcessingCost,
    pub copy_cost: DataProcessingCost,
    pub block_inclusion_window:u64,
}

pub const CONFIG: Configuration = Configuration {
    max_stack_depth:2048,
    max_frame_depth:512,
    max_heap_size:512 * 1024,
    max_bundle_size: 128 * 1024,
    max_txt_alloc: 256 * 64 * 1024,
    max_structural_dept:64,
    return_stack: 256,
    bundle_base_cost: 0,
    entry_store_cost: STORE_WRITE_AND_ENCODE_COST,
    entry_load_cost: STORE_LOAD_AND_ENCODE_COST,
    txt_desc_load_cost: STORE_LOAD_COST,
    parsing_cost: ENCODING_COST,
    copy_cost: COPYING_COST,
    block_inclusion_window: 100,
};

impl Configuration {
    pub const fn calc_heap_size(&self, virt_factor:usize) -> usize {
        Heap::elems::<Entry>(self.max_stack_depth)
            + Heap::elems::<Frame>(self.max_stack_depth)
            + Heap::elems::<Entry>(self.return_stack)
            + (self.max_heap_size* virt_factor)
            + (self.max_bundle_size * virt_factor)
            + (self.max_txt_alloc * virt_factor)
    }
}


pub trait TransactionBundle {
    fn byte_size(&self) -> usize;
    fn earliest_block(&self) -> u64;
    fn param_heap_limit(&self) -> u16;
    fn transaction_heap_limit(&self) -> u32;
    fn stack_elem_limit(&self) -> u16;
    fn stack_frame_limit(&self) -> u16;
    fn runtime_heap_limit(&self) -> u16;
    fn essential_gas_cost(&self) -> u64;
    fn total_gas_cost(&self) -> u64;
    fn sections(&self) -> SlicePtr<BundleSection>;
    fn descriptors(&self) -> SlicePtr<Hash>;
    fn scratch_pad_slots(&self) -> u8;
    fn stored(&self) -> SlicePtr<Hash>;
    fn literal(&self) -> SlicePtr<SlicePtr<u8>>;
    fn witness(&self) -> SlicePtr<SlicePtr<u8>>;
}

pub struct Context<'a,'b, S:Store, T:TransactionBundle> {
    pub store:&'a S,
    pub txt_bundle:&'b T
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
pub fn execute<'c, 'd:'c, L: Tracker,SYS:SystemContext<'c>>(ctx:Context<SYS::S, SYS::B>, block_no:u64, heap:&'d Heap, tracker:&mut L) -> Result<()> {
    //Check that it is inside limit
    if ctx.txt_bundle.byte_size() > CONFIG.max_bundle_size { return error(||"Transaction Bundle to big")}
    verify_repeated::<SYS>( &ctx, block_no)?;
    verify_once::<SYS>(&SYS::VC::new(), &ctx, heap)?;
    execute_once::<_,SYS>(&SYS::EC::new(), &ctx, block_no, heap, tracker)?;
    Ok(())
}

#[cfg(feature = "deployer")]
pub fn deploy<'c, S:Store, CE:CompilationExternals>(store:&S, deploy_data:&[u8], heap:&Heap, system_mode_on:bool) -> Result<Hash> {
    //Check that it is inside limit
    if deploy_data.len() > CONFIG.max_bundle_size { return error(||"Transaction Bundle to big")}
    //Static allocations (could be done once)
    // A buffer to parse the transaction and load values from store
    let deploy_txt_alloc = heap.new_virtual_arena(CONFIG.max_txt_alloc);
    //Parse the transaction
    let deploy_txt:DeployTransaction = Parser::parse_fully(deploy_data, CONFIG.max_structural_dept, &deploy_txt_alloc)?;

    Ok(match deploy_txt.typ {
        DeployType::Module => {
            //todo: I do not like the to_vec here (as we have it in memory twice now)
            //but without having seperate Transaction type it is hard not to do this
            //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
            let res = deploy_module(store, deploy_txt.data.to_vec(), system_mode_on, true)?;
            store.commit(StorageClass::Module);
            res
        },
        DeployType::Transaction => {
            //todo: I do not like the to_vec here (as we have it in memory twice now)
            //but without having seperate Transaction type it is hard not to do this
            //todo: we may consider passing &[u8] into store and copy there if necessary (but this gives lifetime hell)
            let target = deploy_function(store, deploy_txt.data.to_vec(), true)?;
            let (res,_) = compile_function::<_,CE>(store, target, true)?;
            store.commit(StorageClass::Transaction);
            store.commit(StorageClass::Descriptor);
            res
        }
    })
}