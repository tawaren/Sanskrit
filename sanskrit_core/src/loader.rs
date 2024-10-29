use crate::model::resolved::*;
use core::cell::Cell;
use alloc::vec::Vec;
use crate::resolver::Context;
use sanskrit_common::model::*;
use crate::model::*;
use crate::model::linking::{Component, FastModuleLink};
use sp1_zkvm_col::arena::URef;
use sp1_zkvm_col::DefaultIndexType;
use crate::model::provider::fast_link;

pub type ResolvedCtrs = Vec<Vec<URef<'static,ResolvedType>>>;

pub trait StateManager {
    fn get_link_index(&self, hash:&FastModuleLink) -> DefaultIndexType;
    fn link_from_index(&self, index: DefaultIndexType) ->  URef<'static, ModuleLink>;
    fn get_unique_module(&self, hash: URef<'static, ModuleLink>) -> URef<'static,Module>;
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

pub trait NoUnconstraine:StateManager {
    type U:StateManager;
    fn no_unconstrained(&self) -> Self::U;
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

/*
pub struct FetchCache<T:Component> {
    module:URef<'static, Module>,         //The Corresponding cached Module
    link:FastModuleLink,                  //The Module in link form
    offset:u8,                            //The offset of the Component
    phantom: PhantomData<*const T>,
}
*/
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

    #[inline(always)]
    pub fn is_this_module(&self, target:&FastModuleLink) -> bool {
        self.this.as_ref().is_some_and(|this|this==target)
    }

    #[inline(always)]
    pub fn is_local_type(&self, target:URef<'static,ResolvedType>) -> bool {
        self.this.as_ref().is_some_and(|this|target.is_defining_module(this))
    }

    #[inline(always)]
    pub fn borrow_component<'a, C:Component+'a>(&self, link:&FastModuleLink, offset:u8) -> &'a C {
        let module: &'a Module = link.load(self).to_ref();
        //assert!((offset as usize) < C::num_elems(&module)); <-- get would panic if this wa true
        assert!(!self.is_this_module(link) || (offset as usize) < C::get_local_limit(self));
        C::get(module, offset)
    }

    //Create a local context for it (but from the importers view -- meaning they are from a remote Module and the imported functions are ignored and the applies are substituted)
    pub fn substituted_context<'a>(&'a self, link: &FastModuleLink, import: &PublicImport,  subs:&[URef<'static,ResolvedType>]) -> Context<'a,S> {
        //Generate a local context
        Context::create_and_resolve(&[
            Imports::Module(link),
            Imports::Generics(subs),
            Imports::Public(import),
        ], self)
    }

    //Get the module
    #[inline(always)]
    pub fn get_link_index(&self, link:&FastModuleLink) -> DefaultIndexType {
        self.store.get_link_index(link)
    }

    #[inline(always)]
    pub fn link_from_index(&self, index: DefaultIndexType) -> FastModuleLink{
        let link = self.store.link_from_index(index);
        fast_link(link)
    }

    //Get the module
    #[inline(always)]
    pub fn get_module(&self, link:URef<'static, ModuleLink>) -> URef<'static, Module>{
        self.store.get_unique_module(link)
    }

    #[inline(always)]
    pub fn create_generic_type(&self, gen: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.create_generic_type(gen)
    }

    #[inline(always)]
    pub fn sig_type_dedup(&self, sig: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.sig_type_dedup(sig)
    }

    #[inline(always)]
    pub fn virtual_type_dedup(&self, virt: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.virtual_type_dedup(virt)
    }

    #[inline(always)]
    pub fn projection_type_dedup(&self, proj: ResolvedType) -> URef<'static,ResolvedType> {
        self.store.projection_type_dedup(proj)
    }

    #[inline(always)]
    pub fn data_type_dedup(&self, param:ResolvedComponent, extra:&DataComponent) -> URef<'static,ResolvedType> {
        self.store.data_type_dedup(param, extra)
    }

    #[inline(always)]
    pub fn dedup_callable(&self, call: ResolvedCallable) -> URef<'static,ResolvedCallable> {
        self.store.dedup_callable(call)
    }

    #[inline(always)]
    pub fn dedup_permission(&self, perm: ResolvedPermission) -> URef<'static,ResolvedPermission> {
        self.store.dedup_permission(perm)
    }

    #[inline(always)]
    pub fn dedup_signature(&self, sig: ResolvedSignature) -> URef<'static,ResolvedSignature> {
        self.store.dedup_signature(sig)
    }

    #[inline(always)]
    pub fn dedup_ctr(&self, ctr: ResolvedCtrs) -> URef<'static,ResolvedCtrs> {
        self.store.dedup_ctr(ctr)
    }

    pub fn no_unconstrained(&self) -> Loader<S::U> where S: NoUnconstraine
    {
        Loader {
            store: self.store.no_unconstrained(),
            this: self.this,
            this_deployed_data: Cell::new(self.this_deployed_data.get()),
            this_deployed_sigs: Cell::new(self.this_deployed_sigs.get()),
            this_deployed_functions: Cell::new(self.this_deployed_functions.get()),
            this_deployed_implements: Cell::new(self.this_deployed_implements.get()),
        }
    }
}