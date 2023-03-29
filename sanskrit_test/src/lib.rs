#![feature(test)]
#![feature(associated_type_defaults)]

extern crate sanskrit_test_script_compiler;
extern crate sanskrit_deploy;
extern crate sanskrit_core;
extern crate sanskrit_compile;
extern crate sanskrit_interpreter;
extern crate sanskrit_runtime;
extern crate sanskrit_common;
extern crate sanskrit_memory_store;
extern crate ed25519_dalek;
extern crate sha2;
extern crate rand;
extern crate test;
extern crate wasmi;
extern crate wasmer;
#[macro_use]
extern crate lazy_static;

pub mod externals;
pub mod script_test;
pub mod linear_type_stack_test;
pub mod transaction_test;
