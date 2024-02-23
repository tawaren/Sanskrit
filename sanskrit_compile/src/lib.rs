//#![no_std]

extern crate alloc;

extern crate sanskrit_core;
extern crate sanskrit_interpreter;
extern crate sanskrit_common;

mod collector;
mod compacting;
pub mod compiler;
mod gas_table;
pub mod externals;

use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use sanskrit_common::arena::Heap;
use alloc::vec::Vec;
use externals::CompilationExternals;

//compiles a single top function
pub fn compile_function<S:Store, CE:CompilationExternals>(store:&S, function_hash:Hash, auto_commit:bool) -> Result<(Hash, usize)>{
    //create it
    let (key, data) = create_descriptor::<_,CE>(store, function_hash)?;
    //result size
    let size = data.len();
    //we ignore if it is already in
    //if !store.contains(StorageClass::Descriptor, &key){
    //store it
    store.set(StorageClass::Descriptor, key, data)?;
    if auto_commit {
        store.commit(StorageClass::Descriptor);
    }
    //}

    Ok((key, size))
}

pub fn create_descriptor<S:Store, CE:CompilationExternals>(store:&S, function_hash:Hash) -> Result<(Hash, Vec<u8>)>{
    let heap = Heap::new(10000,4.0);
    let alloc = heap.new_arena(10000);
    //compiles the content
    let txt_desc = compiler::compile_transaction::<S, CE>(&function_hash, store, &alloc)?;
    //serializes the content
    let data = Serializer::serialize_fully(&txt_desc, usize::MAX)?;
    //calcs the Key for the store
    let key = store_hash(&[&data]);
    Ok((key, data))
}

