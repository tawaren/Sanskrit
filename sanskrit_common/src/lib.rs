//This contains the core input structure
//This contains ways to navigate that structure
//This contains way to store & Load stuff

#![no_std]

//extern crate blake3;
extern crate sha2;
extern crate sp1_zkvm_col;
extern crate byteorder;
extern crate alloc;
#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate sanskrit_derive;

pub mod encoding;
pub mod model;
pub mod hashing;
pub mod utils;
