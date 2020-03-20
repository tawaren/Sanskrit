#![no_std]
#![feature(nll)]
#![feature(const_if_match)]

extern crate byteorder;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;
#[macro_use]
extern crate alloc;

pub mod interpreter;
pub mod model;
pub mod externals;
pub mod value_encoding;
#[macro_use]
extern crate lazy_static;