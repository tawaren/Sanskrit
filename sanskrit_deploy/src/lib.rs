#![no_std]
#![feature(nll)]

#[macro_use]
extern crate alloc;
extern crate sanskrit_common;
extern crate sanskrit_core;

#[cfg(feature = "test")]
pub mod linear_type_stack;

#[cfg(not(feature = "test"))]
mod linear_type_stack;
mod validate;
mod native;
mod code_type_checker;


use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use alloc::vec::Vec;
use sanskrit_common::model::*;

//Parses a Data Stream as Module, Validates it and if it checks out deployes it
pub fn deploy_module<S:Store>(store:&S, data:Vec<u8>) -> Result<Hash>{
    //thread::spawn(move || {

    //calcs the ModuleHash
    let module_hash = store_hash(&[&data]);
    //validates the input
    validate::validate(&data, store, module_hash)?;
    //stores the input
    store.set(StorageClass::Module, module_hash,data)?;
    Ok(module_hash)
    //}).join().unwrap();
}
