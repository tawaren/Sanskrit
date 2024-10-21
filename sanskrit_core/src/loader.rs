use sanskrit_common::errors::*;
use crate::model::resolved::*;
use core::cell::Cell;
use alloc::vec::Vec;
use crate::resolver::Context;
use sanskrit_common::model::*;
use crate::model::*;
use crate::model::linking::{Component, FastModuleLink};
use core::marker::PhantomData;
use sp1_zkvm_col::arena::URef;

pub type ResolvedCtrs = Vec<Vec<URef<'static,ResolvedType>>>;

pub trait StateManager {
    fn get_unique_module(&self, hash: URef<'static, ModuleLink>) -> Result<URef<'static,Module>>;
    fn create_generic_type(&self, gen:ResolvedType) -> URef<'static,ResolvedType>;
    fn sig_type_dedup(&self, sig:ResolvedType) -> URef<'static,ResolvedType>;
    fn virtual_type_dedup(&self, virt:ResolvedType) -> URef<'static,ResolvedType>;
    fn projection_type_dedup(&self, proj:ResolvedType) -> URef<'static,ResolvedType>;
    fn data_type_dedup(&self, param:ResolvedComponent, extra:&DataComponent) -> URef<'static,ResolvedType>;
    fn dedup_callable(&self, call:ResolvedCallable) -> URef<'static,ResolvedCallable>;
    fn dedup_permission(&self, perm:ResolvedPermission) -> URef<'static,ResolvedPermission>;
    fn dedup_signature(&self, sig:ResolvedSignature) -> URef<'static,ResolvedSignature>;
    fn dedup_ctr(&self, ctr:ResolvedCtrs) -> URef<'static,ResolvedCtrs>;
}

pub struct Loader<S:StateManager> {
    //The backing global Module store
    store:S, // a reference to the store in case a module is not cached
    // The current Module
    this:Option<FastModuleLink>,
    //Helper to simulate partially loaded modules (necessary to detect cycles in component dependencies)
    //could be removed if turing completeness is required (but then tail call needed to prevent stack depth problems)
    pub this_deployed_data:Cell<usize>,
    pub this_deployed_sigs:Cell<usize>,
    pub this_deployed_functions:Cell<usize>,
    pub this_deployed_implements:Cell<usize>,

}

pub struct FetchCache<T:Component> {
    module:URef<'static, Module>,         //The Corresponding cached Module
    link:FastModuleLink,        //The Module in link form
    offset:u8,                  //The offset of the Component
    phantom: PhantomData<*const T>,
}

impl<S:StateManager> Loader<S> {
    //A new partially loaded Storage Cache
    //it starts out with the module currently processed
    pub fn new_for_module(store:S, module: FastModuleLink) -> Self{
        Loader {
            //Create with 0 available components
            store,
            this: Some(module), //transactions are not in a module
            this_deployed_data: Cell::new(0),
            this_deployed_sigs: Cell::new(0),
            this_deployed_functions: Cell::new(0),
            this_deployed_implements: Cell::new(0),
        }
    }

    //A new fully loaded storage cache
    pub fn new_for_transaction(store:S) -> Self{
        //Works As: current need to use this and all other that can be used from this can not use this
        Loader {
            store,
            this: None, //transactions are not in a module
            this_deployed_data: Cell::new(usize::MAX),
            this_deployed_sigs: Cell::new(usize::MAX),
            this_deployed_functions: Cell::new(usize::MAX),
            this_deployed_implements: Cell::new(usize::MAX),
        }
    }

    pub fn is_this_module(&self, target:&FastModuleLink) -> bool {
        self.this.as_ref().is_some_and(|this|this==target)
    }

    pub fn is_local_type(&self, target:URef<'static,ResolvedType>) -> bool {
        self.this.as_ref().is_some_and(|this|target.is_defining_module(this))
    }

    //Gets a component
    pub fn get_component<C:Component>(&self, link:&FastModuleLink, offset:u8) -> Result<FetchCache<C>> {
        //Get the Module
        let module = link.load(self)?;
        //Check if really their

        if offset as usize >= C::num_elems(&module) {
            return error(||"Linked component is not available")
        }
        if self.is_this_module(link) && offset as usize >= C::get_local_limit(self) {
            return error(||"Linked component is not available")
        }
        //Extract the Adt Cache
        Ok(FetchCache {
            module,
            link: link.clone(),
            offset,
            phantom: PhantomData,
        })
    }

    //Get the module
    pub fn get_module(&self, link:URef<'static, ModuleLink>) -> Result<URef<'static, Module>>{
        self.store.get_unique_module(link)
    }

    pub fn create_generic_type(&self, gen: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.create_generic_type(gen)
    }

    pub fn sig_type_dedup(&self, sig: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.sig_type_dedup(sig)
    }

    pub fn virtual_type_dedup(&self, virt: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.virtual_type_dedup(virt)
    }

    pub fn projection_type_dedup(&self, proj: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.projection_type_dedup(proj)
    }

    pub fn data_type_dedup(&self, param:ResolvedComponent, extra:&DataComponent) -> URef<'static,ResolvedType> {
        self.store.data_type_dedup(param, extra)
    }

    pub fn dedup_callable(&self, call: ResolvedCallable) -> URef<'static,ResolvedCallable> {
        self.store.dedup_callable(call)
    }

    pub fn dedup_permission(&self, perm: ResolvedPermission) -> URef<'static,ResolvedPermission> {
        self.store.dedup_permission(perm)
    }

    pub fn dedup_signature(&self, sig: ResolvedSignature) -> URef<'static,ResolvedSignature> {
        self.store.dedup_signature(sig)
    }

    pub fn dedup_ctr(&self, ctr: ResolvedCtrs) -> URef<'static,ResolvedCtrs> {
        self.store.dedup_ctr(ctr)
    }
}

impl<T:Component> FetchCache<T> {
    //Borrows the Entry
    pub fn retrieve(&self) -> &T {
        &T::get(&self.module, self.offset)
    }
    //gets the modules link
    pub fn module(&self) -> &FastModuleLink {
        &self.link
    }
    //gets the offset
    pub fn offset(&self) -> u8 {
        self.offset
    }
    //Create a local context for it (but from the importers view -- meaning they are from a remote Module and the imported functions are ignored and the applies are substituted)
    pub fn substituted_context<'a, S:StateManager>(&'a self, subs:&[URef<'static,ResolvedType>], store:&'a Loader<S>) -> Result<Context<S>> {
        //Generate a local context
        Context::create_and_resolve(&[
            Imports::Module(&self.link),
            Imports::Generics(subs),
            Imports::Public(self.retrieve().get_public_import()),
        ], store)
    }
}