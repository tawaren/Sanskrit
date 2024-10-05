#![no_std]
extern crate sanskrit_deploy;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate core;
extern crate alloc;

mod pre_load_store;

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use sanskrit_deploy::{deploy_stored_module, validate_function};
use sanskrit_common::errors::*;
use sanskrit_common::model::Hash;
use sanskrit_core::model::Module;

use sanskrit_common::store::{CachedStore, StorageClass, store_hash};
use crate::pre_load_store::PreStore;
use sanskrit_common::encoding::*;

#[macro_use]
extern crate sanskrit_derive;

#[derive(Serializable, Parsable)]
pub struct ValidatedModule {
    pub system_module:bool,
    pub module_hash:Hash
}

#[derive(Serializable, Parsable)]
pub struct ValidatedTransaction {
    pub transaction_hash:Hash
    //Todo: add code wants we do compile?
}

#[derive(Serializable, Parsable)]
pub struct Validation {
    pub system_mode:bool,
    pub modules:Vec<ValidatedModule>,
    pub transactions:Vec<ValidatedTransaction>,
    pub open_dependencies:Vec<Hash>
}


pub fn process_preloaded_deploy(modules:Vec<Vec<u8>>, transactions:Vec<Vec<u8>>, deps:Vec<Vec<u8>>, system_mode_on:bool) -> Result<Validation>{
    let mut store = CachedStore::<Module,PreStore>::new(PreStore::new(), StorageClass::Module);
    let mut mod_compiles = Vec::with_capacity(modules.len());
    let mut txt_compiles = Vec::with_capacity(transactions.len());

    let mut open_deps = BTreeSet::new();

    for m in modules {
        mod_compiles.push(ValidatedModule{
            system_module: m[0] != 0,
            module_hash: store.add_module(m)
        });
    }

    for d in deps {
        let hash = store.add_module(d);
        open_deps.insert(hash);
    }

    for h in &mod_compiles {
        deploy_stored_module(&store,h.module_hash.clone(),system_mode_on)?;
    }

    for h in transactions {
        validate_function(&store,&h )?;
        let transaction_hash = store_hash(&[&h]);
        txt_compiles.push(ValidatedTransaction{transaction_hash})
    }

    Ok(Validation{
        system_mode: system_mode_on,
        modules: mod_compiles,
        transactions: txt_compiles,
        open_dependencies: open_deps.into_iter().collect(),
    })
}
