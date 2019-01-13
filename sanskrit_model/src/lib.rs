#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate alloc;
extern crate sanskrit_common;

#[macro_use]
extern crate sanskrit_derive;

pub mod model;
pub mod context;
