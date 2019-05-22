use sanskrit_common::errors::*;
use model::resolved::*;
use core::cell::RefCell;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::Cell;
use utils::Crc;
use alloc::vec::Vec;
use sanskrit_common::store::*;
use resolver::Cachings;
use resolver::Context;
use sanskrit_common::model::*;
use sanskrit_common::encoding::NoCustomAlloc;
use model::*;
use context::*;

pub struct StorageCache<'a, S:Store + 'a> {
    //Caches the modules
    modules:RefCell<BTreeMap<Hash,Rc<Module>>>,
    // a reference to the store in case a module is not cached
    store:&'a S,
    //Helper to simulate partially loaded modules (necessary to detect cycles in component dependencies)
    //could be removed if turing copleteness is required (but then tail call needed to prevent stack depth problems)
    pub this_deployed_adts:Cell<usize>,
    pub this_deployed_functions:Cell<usize>,
}

//This is a helper that allows to Return a cache entry so that others can borrow from it
//necessary as else the cache becomes unusable because it is borrowed mutably
pub struct AdtCache {
    module:Rc<Module>,      //The Corresponding cached Module
    link:ModuleLink,        //The Module in link form
    offset:u8,              //The offset of the Adt
}

//This is a helper that allows to Return a cache entry so that others can borrow from it
//necessary as else the cache becomes unusable because it is borrowed mutably
pub struct FuncCache {
    module:Rc<Module>,  //The Corresponding cached Module
    link:ModuleLink,        //The Module in link form
    offset:u8,              //The offset of the Function
}

impl<'a, S:Store + 'a> StorageCache<'a,S> {

    //A new partially loaded Storage Cache
    //it starts out with the module currently processed
    pub fn new_incremental(store:&'a S,  link:Hash, module:Module) -> Self{
        //A new empty cache
        let mut modules = BTreeMap::new();
        //Insert the Module
        modules.insert(link,Rc::new(module));

        //Create the Storage Cache with 0 avaiable components
        StorageCache{
            modules:RefCell::new(modules),
            store,
            this_deployed_adts: Cell::new(0),
            this_deployed_functions: Cell::new(0),
        }

    }

    //A new fully loaded storage cache
    pub fn new_complete(store:&'a S) -> Self{
        //Works As: current need to use this and all other that can be used from this can not use this
        StorageCache{
            modules:RefCell::new(BTreeMap::new()),
            store,
            this_deployed_adts: Cell::new(<usize>::max_value()),            //as modules have max 255 arts this is ok
            this_deployed_functions:  Cell::new(<usize>::max_value()),   //as modules have max 255 arts this is ok
        }
    }

    //Get the cache of a Module
    fn get_chached_module(&self, hash:&Hash) -> Result<Rc<Module>>{
        //Borrow the cache
        let mut modules = self.modules.borrow_mut();
        //if already their ignore it else create it
        if !modules.contains_key(hash) {
            //get the module from the store by its hash
            let module = self.store.parsed_get::<Module,NoCustomAlloc>(StorageClass::Module,hash, usize::max_value(), &NoCustomAlloc())?;
            //Ref count it and insert it
            let res = Rc::new(module);
            modules.insert(hash.clone(),res.clone());
            Ok(res)
        } else {
            //just use the existing
            Ok(modules[hash].clone())
        }
    }

    //Get the module
    pub fn get_module(&self, link:&Hash) -> Result<Rc<Module>>{
        //Get the Module and extract the module from it
        Ok(self.get_chached_module(link)?.clone())
    }

    //Get an Adt
    pub fn get_adt(&self, link:&ModuleLink, offset:u8) -> Result<AdtCache>{
        //Get the Module
        let module = self.get_chached_module(&link.to_hash())?;
        //Check if really their
        if offset as usize >= module.adts.len() {return item_not_found()}
        //Extract the Adt Cache
        Ok(AdtCache {
            module,
            link: *link,
            offset,
        })
    }

    //Get a Function
    pub fn get_func(&self, link:&ModuleLink, offset:u8) -> Result<FuncCache>{
        //Get the Module
        let module = self.get_chached_module(&link.to_hash())?;
        //Check if really their
        if offset as usize >= module.functions.len() {return item_not_found()}
        //Extract the Fun Cache
        Ok(FuncCache {
            module,
            link:*link,
            offset,
        })
    }
}

impl AdtCache {
    //Borrows the Entry
    pub fn retrieve(&self) -> &AdtComponent {
        &self.module.adts[self.offset as usize]
    }
    //gets the modules link
    pub fn module(&self) -> ModuleLink {
        self.link
    }
    //Create a local context for it (but from the importers view -- meaning they are from a remote Module and the imported functions are ignored and the applies are substituted)
    pub fn substituted_context<'a,'b:'a,S:Store+'b>(&'a self, subs:Vec<Crc<ResolvedType>>, store:&'b StorageCache<'b,S>) -> Result<Context<'a,'b,S>> {
        //Create the input Context
        let ctx = InputContext::from_adt_import(self.retrieve(), self.link);
        //Fetch/create the local cache
        let cache = Rc::new(Cachings::new(&ctx));
        //Create the context
        Ok(Context {
            ctx,
            subs,
            cache,
            store,
            checking:false, //if it is in the store, then it is already checked
        })
    }
}


impl FuncCache {
    //Borrows the Entry
    pub fn retrieve(&self) -> &FunctionComponent {
        &self.module.functions[self.offset as usize]
    }
    //gets the modules link
    pub fn module(&self) -> ModuleLink {
        self.link
    }
    //Create a local context for it (but from the importers view -- meaning they are from a remote Module and the imported functions are ignored and the applies are substituted)
    pub fn substituted_context<'a,'b:'a,S:Store+'b>(&'a self, subs:Vec<Crc<ResolvedType>>, store:&'b StorageCache<'b,S>) -> Result<Context<'a,'b,S>> {
        //Create the input Context
        let ctx = InputContext::from_fun_import(self.retrieve(), self.link);
        //Fetch/create the local cache
        let cache = Rc::new(Cachings::new(&ctx));
        //Create the context
        Ok(Context {
            ctx,
            subs,
            cache,
            store,
            checking:false, //if it is in the store, then it is already checked
        })
    }
}
