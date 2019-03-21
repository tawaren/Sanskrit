#![no_std]
#![feature(nll)]

extern crate byteorder;
extern crate num_traits;
extern crate ed25519_dalek;
extern crate sha2;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;

pub mod interpreter;
pub mod model;
