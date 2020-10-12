#![no_std]
#![feature(nll)]

extern crate alloc;

extern crate sanskrit_core;
extern crate sanskrit_interpreter;
extern crate sanskrit_common;

pub mod limiter;
mod collector;
mod compacting;
pub mod compiler;
mod gas_table;

use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use sanskrit_interpreter::externals::CompilationExternals;
use sanskrit_core::accounting::Accounting;
use limiter::Limiter;
use sanskrit_common::arena::Heap;
use alloc::vec::Vec;

//compiles a single top function
pub fn compile_function<S:Store, E:CompilationExternals>(store:&S, accounting:&Accounting, limiter:&Limiter, function_hash:Hash, auto_commit:bool) -> Result<(Hash, usize)>{
    //create it
    let (key, data) = create_descriptor::<S,E>(store, accounting, limiter, function_hash)?;
    //result size
    let size = data.len();
    //we ignore if it is already in
    if !store.contains(StorageClass::Descriptor, &key){
        //store it
        store.set(StorageClass::Descriptor, key, data)?;
        if auto_commit {
            store.commit(StorageClass::Descriptor);
        }
    }

    Ok((key, size))
}

pub fn create_descriptor<S:Store, E:CompilationExternals>(store:&S, accounting:&Accounting, limiter:&Limiter, function_hash:Hash) -> Result<(Hash, Vec<u8>)>{
    let heap = Heap::new(10000,4.0);
    let alloc = heap.new_arena(10000);
    //compiles the content
    let txt_desc = compiler::compile_transaction::<S, E>(&function_hash, store, accounting, limiter, &alloc)?;
    //serializes the content
    let data = Serializer::serialize_fully(&txt_desc, usize::max_value())?;
    //calcs the Key for the store
    let key = store_hash(&[&data]);
    Ok((key, data))
}

