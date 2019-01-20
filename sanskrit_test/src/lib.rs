#![feature(test)]

extern crate sanskrit_test_script_compiler;
extern crate sanskrit_deploy;
extern crate sanskrit_core;
extern crate sanskrit_compile;
extern crate sanskrit_runtime;
extern crate sanskrit_common;
extern crate sanskrit_memory_store;

extern crate test;

pub mod script_test;
pub mod linear_type_stack_test;
pub mod transaction_test;
#[cfg(test)]
pub mod gas_bench;