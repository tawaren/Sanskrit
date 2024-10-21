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

    //println!("cycle-tracker-report-start: loading");
    let system_mode = sp1_zkvm::io::read::<bool>();
    let modules = load_vec();
    let transactions = load_vec();
    let dependencies = load_vec();
    //println!("cycle-tracker-report-end: loading");

    match process_preloaded_deploy(modules, transactions, dependencies, system_mode) {
        Ok(hs) => {
            let ser = Serializer::serialize_fully(&hs).expect("serialisation failed");
            //println!("DeDup Counter was: {}", unsafe{dedup_count});
            //println!("DeDup Counter fresh was: {}", unsafe{dedup_miss_count});
            //println!("DeDup Max size was: {}", unsafe{dedup_max});

            sp1_zkvm::io::commit_slice(&ser)
        },
        Err(e) => {
            println!("{:?}", e);
            assert!(false)
        }
    }
}
