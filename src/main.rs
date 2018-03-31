//clippy = "*"
//#![feature(plugin)]
//#![plugin(clippy)]
#![feature(try_trait)]
#![feature(const_fn)]
#![feature(catch_expr)]

extern crate blake2_rfc;
extern crate lmdb_rs as lmdb;
#[macro_use]
extern crate lazy_static;
extern crate constant_time_eq;
extern crate byteorder;
extern crate lazycell;


pub mod compiler;
pub mod test;


fn main() {

}