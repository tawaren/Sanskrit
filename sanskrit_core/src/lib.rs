#![no_std]
#![feature(alloc)]
#![feature(nll)]

#[macro_use]
extern crate alloc;
extern crate sanskrit_common;
#[macro_use]
extern crate sanskrit_derive;

pub mod utils;
pub mod loader;
pub mod resolver;
pub mod native;
pub mod context;
pub mod model;
