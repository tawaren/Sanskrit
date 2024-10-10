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

use sanskrit_common::errors::*;
use sanskrit_common::encoding::Parser;
use sanskrit_common::model::*;
use sanskrit_common::supplier::{store_hash, Supplier};
use sanskrit_core::model::{FunctionComponent, Module};
use sanskrit_core::model::linking::{FastModuleLink, Link};


pub fn validate_stored_module<S:Supplier<Module>>(store:&S, link:FastModuleLink, system_mode_allowed:bool) -> Result<()>{
    assert!(link.is_local_link());
    //validates the input
    validate::validate(store, link, system_mode_allowed)
}

//Processes a function used by compiler to check top level transactions
pub fn validate_unparsed_function<S:Supplier<Module>>(store:&S, data:&[u8]) -> Result<Hash>{
    //calcs the FunctionHash
    let function_hash = store_hash(&[data]);
    //Parse the function
    let fun:FunctionComponent = Parser::parse_fully::<FunctionComponent>(data)?;
    //if it is already deployed we can ignore it
    //validates the input
    validate::validate_top_function(fun, store)?;
    Ok(function_hash)
}