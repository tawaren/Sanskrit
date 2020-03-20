#![feature(test)]

extern crate sanskrit_deploy;
extern crate sanskrit_core;
extern crate sanskrit_compile;
extern crate sanskrit_interpreter;
extern crate sanskrit_runtime;
extern crate sanskrit_common;
extern crate sanskrit_memory_store;

extern crate byteorder;
extern crate hex;
extern crate itertools;
extern crate test;
#[macro_use]
extern crate arrayref;
extern crate ed25519_dalek;
extern crate sha2;
extern crate rand;

#[macro_use] extern crate lalrpop_util;

pub mod environment;
pub mod generate;
pub mod model;
pub mod script;

lalrpop_mod!(pub parser);
