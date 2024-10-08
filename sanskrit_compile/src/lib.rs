#![no_std]

extern crate alloc;

extern crate sanskrit_core;
extern crate sanskrit_chain_code;
extern crate sanskrit_common;

mod collector;
mod compacting;
pub mod compiler;
pub mod externals;

use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use alloc::vec::Vec;
use externals::CompilationExternals;
use sanskrit_core::model::Module;

//compiles a single top function
pub fn compile_function<S:Store, CE:CompilationExternals>(store:&CachedStore<Module,S>, function_hash:Hash, auto_commit:bool) -> Result<(Hash, usize)>{
    //create it
    let (key, data) = create_descriptor::<_,CE>(store, function_hash)?;
    //result size
    let size = data.len();
    //we ignore if it is already in

    //store it
    match store.set(StorageClass::Descriptor, key, data) {
        Ok(_) => {}
        //Todo: We ignore for now if it is already in the store
        Err(_) => {}

    }
    if auto_commit {
        store.commit(StorageClass::Descriptor);
    }


    Ok((key, size))
}

pub fn create_descriptor<S:Store, CE:CompilationExternals>(store:&CachedStore<Module,S>, function_hash:Hash) -> Result<(Hash, Vec<u8>)>{
    //compiles the content
    let txt_desc = compiler::compile_transaction::<S, CE>(&function_hash, store)?;
    //serializes the content
    let data = Serializer::serialize_fully(&txt_desc)?;
    //calcs the Key for the store
    let key = store_hash(&[&data]);
    Ok((key, data))
}

