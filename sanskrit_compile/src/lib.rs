#![no_std]
#![feature(alloc)]
#![feature(nll)]


extern crate alloc;

extern crate sanskrit_core;
extern crate sanskrit_runtime;
extern crate sanskrit_common;

pub mod compacting;
pub mod compiler;
mod gas_table;

use sanskrit_common::model::*;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use compiler::ComponentProcessor;
use sanskrit_runtime::model::AdtDescriptor;
use sanskrit_runtime::model::FunctionDescriptor;

struct StoreDescriptorProcessor<'a, S:Store> {
    module_hash:Hash,
    store:&'a S
}

impl<'a, S:Store> ComponentProcessor for StoreDescriptorProcessor<'a, S> {
    fn process_adt<'b>(&mut self, offset: u8, a_desc: &AdtDescriptor<'b>) -> Result<()> {
        //calcs the Key for the store
        let key = store_hash(&[&self.module_hash,&[offset]]);
        //serializes the content
        let mut s = Serializer::new(usize::max_value());
        a_desc.serialize(&mut s)?;
        let data = s.extract();
        //stores it
        self.store.set(StorageClass::AdtDesc, key, data)
    }

    fn process_fun<'b>(&mut self, offset: u8, f_desc: &FunctionDescriptor<'b>) -> Result<()> {
        //calcs the Key for the store
        let key = store_hash(&[&self.module_hash,&[offset]]);
        //serializes the content
        let mut s = Serializer::new(usize::max_value());
        f_desc.serialize(&mut s)?;
        let data = s.extract();
        //stores it
        self.store.set(StorageClass::FunDesc, key, data)
    }
}

//compiles a whole module
pub fn compile_module<S:Store>(store:&S, module_hash:Hash) -> Result<()>{
    //todo: ensure that the store makes only a single transaction or stuff may be partially stored
    //todo: Alternative: collect in Vector & store at end

    let mut proc = StoreDescriptorProcessor {module_hash, store};

    //Todo: Start Txt
    //compiles the content
    compiler::compile(&module_hash,store, &mut proc)?;

    //Todo: End Txt

    Ok(())
}
