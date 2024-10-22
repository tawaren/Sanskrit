#![no_std]
extern crate sanskrit_deploy;
extern crate sanskrit_common;
extern crate sanskrit_core;
extern crate core;
extern crate alloc;
extern crate sp1_zkvm_col;

use alloc::vec::Vec;
use sanskrit_deploy::{validate_stored_module, validate_unparsed_function};
use sanskrit_common::model::Hash;
use sanskrit_common::encoding::*;
use sanskrit_core::model::linking::FastModuleLink;
use sanskrit_core::model::provider::StaticProvider;
use sp1_zkvm_col::vec::UniqueVecBuilder;

#[macro_use]
extern crate sanskrit_derive;

#[derive(Serializable, Parsable)]
pub struct ValidatedModule {
    pub system_module:bool,
    //Will be Serialized as ModuleLink which serializes as Hash
    pub module_hash:FastModuleLink
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
    //Will be Serialized as Vec<ModuleLink> which serializes as Vec<Hash>
    pub open_dependencies:Vec<FastModuleLink>
}

pub fn process_preloaded_deploy(modules:Vec<Vec<u8>>, transactions:Vec<Vec<u8>>, deps:Vec<Vec<u8>>, system_mode_allowed:bool) -> Validation{
    let mut provider = StaticProvider::new();

    let mut mod_compiles = Vec::with_capacity(modules.len());
    let mut txt_compiles = Vec::with_capacity(transactions.len());
    let mut open_deps = UniqueVecBuilder::with_capacity(deps.len());

    for m in modules {
        mod_compiles.push(ValidatedModule{
            system_module: m[0] != 0,
            module_hash: provider.add(&m)
        });
    }

    for d in deps {
        let hash = provider.add(&d);
        open_deps.add(hash);
    }

    for h in &mod_compiles {
        validate_stored_module(provider, h.module_hash.clone(), system_mode_allowed);
    }

    for h in transactions {
        let transaction_hash = validate_unparsed_function(provider,&h);
        txt_compiles.push(ValidatedTransaction{transaction_hash})
    }

    provider.validate();

    Validation{
        system_mode: system_mode_allowed,
        modules: mod_compiles,
        transactions: txt_compiles,
        open_dependencies: open_deps.finalize(),
    }
}
