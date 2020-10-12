#![no_std]
#![feature(nll)]
#![feature(const_if_match)]

extern crate byteorder;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;
extern crate alloc;
extern crate ed25519_dalek;
extern crate sha2;

pub mod interpreter;
pub mod model;
pub mod externals;
pub mod value_encoding;
