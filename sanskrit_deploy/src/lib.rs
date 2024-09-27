#![no_std]

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

//Todo: Make Configurable
const INPUT_SIZE_LIMIT:usize = 256000;
//const INPUT_SIZE_LIMIT:usize = 2048000; //for test

pub fn deploy_stored_module<S:Store>(store:&S, module_hash:Hash, system_mode_on:bool) -> Result<()>{
    store.get(StorageClass::Module, &module_hash, |data|{
        inner_deploy_module(store,module_hash,data,system_mode_on)
    })?
}

pub fn deploy_module<S:Store>(store:&S, data:Vec<u8>, system_mode_on:bool, auto_commit:bool) -> Result<Hash>{
    //calcs the ModuleHash
    let module_hash = store_hash(&[&data]);
    inner_deploy_module(store,module_hash,&data,system_mode_on)?;
    //stores the input
    match store.set(StorageClass::Module, module_hash,data) {
        Ok(_) => {}
        //Todo: We ignore for now if it is already in the store
        Err(_) => {}
    }
    if auto_commit {
        store.commit(StorageClass::Module);
    }
    Ok(module_hash)
}

fn inner_deploy_module<S:Store>(store:&S, module_hash:Hash, data:&[u8], system_mode_on:bool) -> Result<()>{
    //Check input limitation constraint
    if data.len() > INPUT_SIZE_LIMIT {
        return error(||"Input is to big")
    }
    //if it is already deployed we can ignore it
    //validates the input
    validate::validate(&data, store, module_hash, system_mode_on)?;
    Ok(())
}

//Processes a function used by compiler to check top level transactions
pub fn deploy_function<S:Store>(store:&S, data:Vec<u8>, auto_commit:bool) -> Result<Hash>{
    //Check input limitation constraint
    if data.len() > INPUT_SIZE_LIMIT {
        return error(||"Input is to big")
    }
    //calcs the FunctionHash
    let function_hash = store_hash(&[&data]);
    //if it is already deployed we can ignore it
    //validates the input
    validate::validate_top_function(&data, store)?;
    //stores the input
    match store.set(StorageClass::Transaction, function_hash, data) {
        Ok(_) => {}
        //Todo: We ignore for now if it is already in the store
        Err(_) => {}
    }
    if auto_commit {
        store.commit(StorageClass::Transaction);
    }
    //}
    Ok(function_hash)
}
