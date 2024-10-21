#![no_std]

extern crate alloc;
extern crate sanskrit_common;
extern crate sp1_zkvm_col;
extern crate sp1_zkvm;

#[macro_use]
extern crate sanskrit_derive;

#[cfg(feature = "multi-thread")]
extern crate spin;


pub mod loader;
pub mod resolver;
pub mod model;
