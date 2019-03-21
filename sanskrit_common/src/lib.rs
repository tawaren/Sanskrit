//This contains the core input structure
//This contains ways to navigate that structure
//This contains way to store & Load stuff

#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate blake2_rfc;
extern crate byteorder;
extern crate alloc;
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate sanskrit_derive;


pub mod encoding;
pub mod store;
pub mod errors;
pub mod linear_stack;
pub mod model;
pub mod capabilities;
pub mod arena;
pub mod hashing;