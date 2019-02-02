#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate blake2_rfc;
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate arrayref;
extern crate byteorder;
extern crate num_traits;
extern crate ed25519_dalek;
extern crate sha2;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;

pub mod interpreter;
pub mod model;
pub mod hashing;