//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra sys so that it behaves properly
// inside the zkVM.
//#![no_std]
#![no_main]
extern crate alloc;
sp1_zkvm::entrypoint!(main);

use alloc::vec::Vec;
use sanskrit_preloaded_validation::process_preloaded_deploy;
use sanskrit_common::encoding::*;

fn load_vec() -> Vec<Vec<u8>> {
    let n = sp1_zkvm::io::read::<u32>();
    let mut loaded = Vec::with_capacity(n as usize);
    for _ in 0..n {
        loaded.push(sp1_zkvm::io::read_vec());
    }
    loaded
}

pub fn main() {

    let system_mode = sp1_zkvm::io::read::<bool>();
    let modules = load_vec();
    let transactions = load_vec();
    let dependencies = load_vec();


    match process_preloaded_deploy(modules, transactions, dependencies, system_mode) {
        Ok(hs) => {
            let ser = Serializer::serialize_fully(&hs,usize::MAX).expect("serialisation failed");
            sp1_zkvm::io::commit_slice(&ser)
        },
        Err(e) => {
            println!("{:?}", e);
            assert!(false)
        }
    }
}
