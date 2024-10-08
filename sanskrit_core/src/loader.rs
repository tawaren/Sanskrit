use alloc::collections::BTreeMap;
use sanskrit_common::errors::*;
use crate::model::resolved::*;
use core::cell::RefCell;
use alloc::rc::Rc;
use core::cell::Cell;
use crate::utils::{Crc, CrcDeDup};
use alloc::vec::Vec;
use sanskrit_common::store::*;
use crate::resolver::Context;
use sanskrit_common::model::*;
use crate::model::*;
use crate::model::linking::{Link, Component};
use core::marker::PhantomData;

pub struct Loader<'a, S:Store + 'a> {
    //Caches the modules
    // we treat the module link in this rust implementation as an object but it is not in the specification
    //  the specification specifies the loaded modules instead. We split them in two
    dedup_hash:RefCell<CrcDeDup<ModuleLink>>,
    modules:RefCell<BTreeMap<Hash,Crc<Module>>>,
    store:&'a CachedStore<Module, S>, // a reference to the store in case a module is not cached
    // Object loaders
    // deduplication
    dedup_type:RefCell<CrcDeDup<ResolvedType>>,
    dedup_call:RefCell<CrcDeDup<ResolvedCallable>>,
    dedup_perm:RefCell<CrcDeDup<ResolvedPermission>>,
    dedup_sig:RefCell<CrcDeDup<ResolvedSignature>>,
    dedup_ctr:RefCell<CrcDeDup<Vec<Vec<Crc<ResolvedType>>>>>,
    //Helper to simulate partially loaded modules (necessary to detect cycles in component dependencies)
    //could be removed if turing completeness is required (but then tail call needed to prevent stack depth problems)
    pub this_deployed_data:Cell<usize>,
    pub this_deployed_sigs:Cell<usize>,
    pub this_deployed_functions:Cell<usize>,
    pub this_deployed_implements:Cell<usize>,

}

pub struct FetchCache<T:Component> {
    module:Crc<Module>,         //The Corresponding cached Module
    link:Crc<ModuleLink>,       //The Module in link form
    offset:u8,                  //The offset of the Component
    phantom: PhantomData<*const T>,
}

impl<'a, S:Store + 'a> Loader<'a,S> {
    //A new partially loaded Storage Cache
    //it starts out with the module currently processed
    pub fn new_incremental(store:&'a CachedStore<Module, S>,  link:Hash, module:Rc<Module>) -> Self{
        //A new empty cache
        let mut modules = BTreeMap::new();
        //Insert the Module
        modules.insert(link,Crc{elem:module});

        //Create the Storage Cache with 0 avaiable components
        Loader {
            modules:RefCell::new(modules),
            store,
            dedup_type: RefCell::new(CrcDeDup::new()),
            dedup_call: RefCell::new(CrcDeDup::new()),
            dedup_perm: RefCell::new(CrcDeDup::new()),
            dedup_sig: RefCell::new(CrcDeDup::new()),
            dedup_hash: RefCell::new(CrcDeDup::new()),
            dedup_ctr: RefCell::new(CrcDeDup::new()),
            this_deployed_data: Cell::new(0),
            this_deployed_sigs: Cell::new(0),
            this_deployed_functions: Cell::new(0),
            this_deployed_implements: Cell::new(0),
        }

    }

    //A new fully loaded storage cache
    pub fn new_complete(store:&'a CachedStore<Module, S>) -> Self{
        //Works As: current need to use this and all other that can be used from this can not use this
        Loader {
            modules:RefCell::new(BTreeMap::new()),
            store,
            dedup_type: RefCell::new(CrcDeDup::new()),
            dedup_call: RefCell::new(CrcDeDup::new()),
            dedup_perm: RefCell::new(CrcDeDup::new()),
            dedup_sig: RefCell::new(CrcDeDup::new()),
            dedup_hash: RefCell::new(CrcDeDup::new()),
            dedup_ctr: RefCell::new(CrcDeDup::new()),
            this_deployed_data: Cell::new(<usize>::MAX),          //as modules have max 255 adts this is ok
            this_deployed_sigs: Cell::new(<usize>::MAX),           //as modules have max 255 sigs this is ok
            this_deployed_functions: Cell::new(<usize>::MAX),    //as modules have max 255 funs this is ok
            this_deployed_implements: Cell::new(<usize>::MAX),   //as modules have max 255 impls this is ok
        }
    }

    //Get the cache of a Module
    fn get_cached_module(&self, hash:Hash) -> Result<Crc<Module>>{
        //Borrow the cache
        let mut modules = self.modules.borrow_mut();
        //if already their ignore it else create it
        if !modules.contains_key(&hash) {
            //get the module from the store by its hash
            let module = self.store.get_cached(&hash)?;
            //Ref count it and insert it
            let res = Crc{elem:module};
            modules.insert(hash,res.clone());
            Ok(res)
        } else {
            //just use the existing
            Ok(modules[&hash].clone())
        }
    }

    //Get the module
    pub fn get_module(&self, link:Hash) -> Result<Crc<Module>>{
        //Get the Module and extract the module from it
        Ok(self.get_cached_module(link)?.clone())
    }

    //Gets a component
    pub fn get_component<C:Component>(&self, link:&Crc<ModuleLink>, offset:u8) -> Result<FetchCache<C>> {
        //Get the Module
        let module = self.get_cached_module(link.to_hash())?;
        //Check if really their

        if offset as usize >= C::num_elems(&module) {
            return error(||"Linked component is not available")
        }
        if link.is_local_link() && offset as usize >= C::get_local_limit(self) {
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

    pub fn dedup_module_link(&self, link:ModuleLink) -> Crc<ModuleLink> {
        //module links are not part of the specification so we exclude them from tests (as they are allowed and will be created outside of the loade)
        self.dedup_hash.borrow_mut().dedup(link)
    }

    //Dedup a type
    pub fn dedup_type(&self, typ:ResolvedType) -> Crc<ResolvedType> {
        self.dedup_type.borrow_mut().dedup(typ)
    }

    pub fn dedup_callable(&self, call:ResolvedCallable) -> Crc<ResolvedCallable> {
        self.dedup_call.borrow_mut().dedup(call)
    }

    pub fn dedup_permission(&self, perm:ResolvedPermission) -> Crc<ResolvedPermission> {
        self.dedup_perm.borrow_mut().dedup(perm)
    }

    pub fn dedup_signature(&self, sig:ResolvedSignature) -> Crc<ResolvedSignature> {
        self.dedup_sig.borrow_mut().dedup(sig)
    }

    pub fn dedup_ctr(&self, ctr:Vec<Vec<Crc<ResolvedType>>>) -> Crc<Vec<Vec<Crc<ResolvedType>>>> {
        self.dedup_ctr.borrow_mut().dedup(ctr)
    }

}

impl<T:Component> FetchCache<T> {
    //Borrows the Entry
    pub fn retrieve(&self) -> &T {
        &T::get(&self.module, self.offset)
    }
    //gets the modules link
    pub fn module(&self) -> &Crc<ModuleLink> {
        &self.link
    }
    //gets the offset
    pub fn offset(&self) -> u8 {
        self.offset
    }
    //Create a local context for it (but from the importers view -- meaning they are from a remote Module and the imported functions are ignored and the applies are substituted)
    pub fn substituted_context<'a, 'b:'a,S:Store+'b>(&self, subs:&'a [Crc<ResolvedType>], store:&'b Loader<'b,S>) -> Result<Context<'b,S>> {
        //Generate a local context
        Context::create_and_resolve(&[
            Imports::Module(&self.link),
            Imports::Generics(subs),
            Imports::Public(self.retrieve().get_public_import()),
        ], store)
    }
}