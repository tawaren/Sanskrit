#![no_std]
#![feature(nll)]

#[macro_use]
extern crate alloc;
extern crate sanskrit_common;
extern crate sanskrit_core;

#[cfg(feature = "test")]
pub mod linear_stack;


#[cfg(not(feature = "test"))]
mod linear_stack;

mod validate;
//mod native;
mod code_type_checker;


use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use alloc::vec::Vec;
use sanskrit_common::model::*;
use sanskrit_core::accounting::Accounting;

pub fn deploy_module<S:Store>(store:&S, accounting:&Accounting, data:Vec<u8>, system_mode_on:bool, auto_commit:bool) -> Result<Hash>{
    //thread::spawn(move || {
    //Check input limitation constraint
    if data.len() > accounting.input_limit {
        return error(||"Input is to big")
    }
    //account for the store
    accounting.store_bytes(data.len())?;
    //calcs the ModuleHash
    let module_hash = store_hash(&[&data]);
    //if it is already deployed we can ignore it
    if !store.contains(StorageClass::Module, &module_hash) {
        //validates the input
        validate::validate(&data, store, accounting, module_hash, system_mode_on)?;
        //stores the input
        store.set(StorageClass::Module, module_hash,data)?;
        if auto_commit {
            store.commit(StorageClass::Module);
        }
    }
    Ok(module_hash)
    //}).join().unwrap();
}

//Processes a function used by compiler to check top level transactions
pub fn deploy_function<S:Store>(store:&S, accounting:&Accounting, data:Vec<u8>, auto_commit:bool) -> Result<Hash>{
    //Check input limitation constraint
    if data.len() > accounting.input_limit {
        return error(||"Input is to big")
    }
    //account for the store
    accounting.store_bytes(data.len())?;
    //calcs the FunctionHash
    let function_hash = store_hash(&[&data]);
    //if it is already deployed we can ignore it
    if !store.contains(StorageClass::Transaction, &function_hash) {
        //validates the input
        validate::validate_top_function(&data, store, accounting)?;
        //stores the input
        store.set(StorageClass::Transaction, function_hash, data)?;
        if auto_commit {
            store.commit(StorageClass::Transaction);
        }
    }
    Ok(function_hash)
}
