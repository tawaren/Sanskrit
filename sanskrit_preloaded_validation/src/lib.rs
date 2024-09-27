#![no_std]
extern crate sanskrit_deploy;
extern crate sanskrit_common;
extern crate core;
extern crate alloc;

mod pre_load_store;

use alloc::vec::Vec;
use sanskrit_deploy::{deploy_stored_module};
use sanskrit_common::errors::*;
use sanskrit_common::model::Hash;
use sanskrit_common::store::{Store};
use crate::pre_load_store::PreStore;


pub fn process_preloaded_module_deploy(modules:Vec<Vec<u8>>, deps:Vec<Vec<u8>>, system_mode_on:bool) -> Result<Vec<Hash>>{
    let (store, compiles) = PreStore::new(modules, deps);
    for h in &compiles {
        deploy_stored_module(&store,h.clone(),system_mode_on)?
    }
    Ok(compiles)
}
