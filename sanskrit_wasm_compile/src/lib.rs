#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate pwasm_ethereum;
#[macro_use]
extern crate arrayref;
extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_memory_store; //for now later use an ethereum or substrate based one

use pwasm_ethereum as ext;
use sanskrit_memory_store::BTreeMapStore;
use sanskrit_compile::compile_module;
use sanskrit_common::model::hash_from_slice;

#[no_mangle]
pub fn call() {
    let s = BTreeMapStore::new();
    let input = ext::input();
    compile_module(&s,hash_from_slice(&input)).unwrap();
}

