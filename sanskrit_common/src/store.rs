use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use crate::errors::*;
use crate::encoding::*;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use crate::model::Hash;
use crate::hashing::*;

//Trait representing a store
//Allows it to be flexible from Temporary in Memory, over stateless in Memory to persistent
pub trait Store {
    //Check if something is their
    //fn contains(&self, class:StorageClass, key: &Hash) -> bool;
    //Check if something is their
    fn delete(&self, class:StorageClass, key: &Hash) -> Result<()>;
    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8]) -> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P>;
    //Stores a value in the store (reqiures that it is empty)
    fn set(&self, class:StorageClass, key:Hash, data:Vec<u8>) -> Result<()> ;
    //commits accumulated changes
    fn commit(&self, class:StorageClass);
    //reverts accumulated changes;
    fn rollback(&self, class:StorageClass);

    //helper
    fn parsed_get<'a, P:Parsable<'a>, A: ParserAllocator>(&self, class:StorageClass, key: &Hash, max_dept:usize, alloc:&'a A) -> Result<P>{
       self.get(class, key,|d| Parser::parse_fully::<P,A>(d, max_dept, alloc))?
    }

    fn serialized_set<S:Serializable,>(&self, class:StorageClass, key:Hash, max_dept:usize, data:&S) -> Result<()>{
        self.set(class,key, Serializer::serialize_fully(data,max_dept)?)
    }

}

//enum pointing to different sections in the store
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[repr(u8)]
pub enum StorageClass{
    Module,
    Transaction,
    Descriptor,
    EntryHash,  // hash(type||value)
    EntryValue //Value will only be needed by state providers
}

//Helper to calc the key for a storage slot
pub fn store_hash(data:&[&[u8]]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = Hasher::new();
    //push the data into it
    for d in data {
        context.update(*d);
    }
    //calc the Hash
    context.finalize()
}

pub struct CachedStore<P, S:Store> {
    cache:RefCell<BTreeMap<Hash, Rc<P>>>,
    class:StorageClass,
    store:S,

}

impl<P, S:Store> Deref for CachedStore<P,S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<P, S:Store> DerefMut for CachedStore<P,S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}


impl<P, S:Store> CachedStore<P,S> {
    pub fn new(store:S, class:StorageClass) -> Self{
        CachedStore {
            cache: RefCell::new(BTreeMap::new()),
            class,
            store
        }
    }
}

impl<'a, P:Parsable<'a>, S:Store> CachedStore<P,S>  {

    pub fn get_cached<A: ParserAllocator>(&self, key: &Hash, max_dept:usize, alloc:&'a A) -> Result<Rc<P>>{
        let mut cache = self.cache.borrow_mut();

        if !cache.contains_key(key) {
            let val:Rc<P> = Rc::new(self.store.parsed_get(self.class,key,max_dept,alloc)?);
            cache.insert(key.clone(), val.clone());
            Ok(val)
        } else {
            //just use the existing
            Ok(cache[key].clone())
        }
    }

    pub fn get_direct<A: ParserAllocator>(&self, data:&[u8], key: &Hash, max_dept:usize, alloc:&'a A) -> Result<Rc<P>>{
        let mut cache = self.cache.borrow_mut();
        if !cache.contains_key(key) {
            let parsed: P = Parser::parse_fully(data, max_dept, alloc)?;
            let val:Rc<P>  = Rc::new(parsed);
            cache.insert(key.clone(), val.clone());
            Ok(val)
        } else {
            //just use the existing
            Ok(cache[key].clone())
        }
    }
}