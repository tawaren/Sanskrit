#![no_std]

#[macro_use]
extern crate alloc;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate sp1_zkvm_col;

#[cfg(feature = "test")]
pub mod linear_stack;


#[cfg(not(feature = "test"))]
mod linear_stack;

mod validate;
//mod native;
mod code_type_checker;

use sanskrit_common::encoding::Parser;
use sanskrit_common::model::*;
use sanskrit_common::utils::store_hash;
use sanskrit_core::loader::StateManager;
use sanskrit_core::model::FunctionComponent;
use sanskrit_core::model::linking::FastModuleLink;


pub fn validate_stored_module<S:StateManager>(provider:S, link:FastModuleLink, system_mode_allowed:bool) {
    //validates the input
    validate::validate(provider, link, system_mode_allowed);
}

//Processes a function used by compiler to check top level transactions
pub fn validate_unparsed_function<S:StateManager>(provider:S, data:&[u8]) -> Hash{
    //calcs the FunctionHash
    let function_hash = store_hash(&[data]);
    //Parse the function
    let fun:FunctionComponent = Parser::parse_fully::<FunctionComponent>(data);
    //if it is already deployed we can ignore it
    //validates the input
    validate::validate_top_function(fun, provider);
    function_hash
}