#![no_std]

extern crate alloc;
extern crate sp1_zkvm;
extern crate sp1_zkvm_col;
extern crate sanskrit_core;
extern crate sanskrit_chain_code;
extern crate sanskrit_common;

mod collector;
mod compacting;
pub mod compiler;
pub mod externals;

use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use alloc::vec::Vec;
use externals::CompilationExternals;
use sanskrit_common::utils::store_hash;
use sanskrit_core::loader::{StateManager, NoUnconstraine};
use sanskrit_core::model::FunctionComponent;

//compiles a single top function
pub fn compile_function<S:StateManager+ NoUnconstraine, CE:CompilationExternals>(store:S, fun:FunctionComponent) -> (Hash, Vec<u8>){
    //compiles the content
    let txt_desc = compiler::compile_transaction::<S, CE>(store, fun);
    //serializes the content
    let data = Serializer::serialize_fully(&txt_desc);
    //calcs the Key for the store
    let key = store_hash(&[&data]);
    (key, data)
}

