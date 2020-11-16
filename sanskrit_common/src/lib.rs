//This contains the core input structure
//This contains ways to navigate that structure
//This contains way to store & Load stuff

#![no_std]
#![feature(nll)]

extern crate blake3;
extern crate byteorder;
extern crate alloc;
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate sanskrit_derive;

pub mod encoding;
pub mod store;
pub mod errors;
pub mod model;
pub mod hashing;
pub mod arena;
