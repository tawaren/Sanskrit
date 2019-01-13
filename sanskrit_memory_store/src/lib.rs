#![no_std]
#![feature(alloc)]
#![feature(nll)]

extern crate alloc;
extern crate sanskrit_common;

use alloc::collections::BTreeMap;
use sanskrit_common::model::Hash;
use alloc::prelude::*;
use core::cell::RefCell;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;


//A container for the different storage sections
// represented as BTreeMaps (as they work with alloc and HashMap seem not to)
#[derive(Clone, Default)]
pub struct InnerBTreeMapStore {
    modules:BTreeMap<Hash, Vec<u8>>,    //All the Modules (Serialized)
    adts:BTreeMap<Hash, Vec<u8>>,    //All the Adt descriptors (Serialized)
    funs:BTreeMap<Hash, Vec<u8>>,    //All the Fun Descriptors (Serialized)
    elems:BTreeMap<Hash, Vec<u8>>,    //All the Elements (Serialized)
}

//A BTreeMap backed store for development
#[derive(Clone, Default)]
pub struct BTreeMapStore(RefCell<InnerBTreeMapStore>);

impl BTreeMapStore {
    //creates a single thread enabled store with inner mutability
    pub fn new()-> Self{
        BTreeMapStore(RefCell::new(InnerBTreeMapStore {
            modules: BTreeMap::new(),
            adts: BTreeMap::new(),
            funs: BTreeMap::new(),
            elems: BTreeMap::new(),
        }))
    }

    //for testing
    pub fn clear_section(&self, class:StorageClass) {
        fn process(map:&mut BTreeMap<Hash, Vec<u8>>) {
            map.clear()
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&mut self.0.borrow_mut().adts),
            StorageClass::FunDesc => process(&mut self.0.borrow_mut().funs),
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Elem => process(&mut self.0.borrow_mut().elems),
        }
    }
}

impl Store for BTreeMapStore {
    //Checks if the elem is contained
    fn contains(&self, class: StorageClass, key: &Hash) -> bool {
        fn process(map:&BTreeMap<Hash, Vec<u8>>, key:&Hash) -> bool {
            map.contains_key(key)
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&self.0.borrow().adts,key),
            StorageClass::FunDesc => process(&self.0.borrow().funs,key),
            StorageClass::Module => process(&self.0.borrow().modules,key),
            StorageClass::Elem => process(&self.0.borrow().elems,key),
        }
    }
    //delete a store entry
    fn delete(&self, class: StorageClass, key: &[u8; 20])  -> Result<()>  {
        fn process(map: &mut BTreeMap<Hash, Vec<u8>>, key:&Hash) -> Result<()>  {
            //remove but give an error if not their
            match map.remove(key) {
                None => item_not_found(),
                Some(_) => Ok(()),
            }
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&mut self.0.borrow_mut().adts,key),
            StorageClass::FunDesc => process(&mut self.0.borrow_mut().funs,key),
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key),
            StorageClass::Elem => process(&mut self.0.borrow_mut().elems,key),
        }
    }


    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        fn process<P,F:FnOnce(&[u8])-> P>(map: &BTreeMap<Hash, Vec<u8>>, key:&Hash, f:F) -> Result<P> {
            //get the key if available, else give an error
            match map.get(key) {
                None =>  item_not_found(),
                Some(data) => Ok(f(data)),
            }
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&self.0.borrow().adts,key,f),
            StorageClass::FunDesc => process(&self.0.borrow().funs,key,f),
            StorageClass::Module => process(&self.0.borrow().modules,key,f),
            StorageClass::Elem => process(&self.0.borrow().elems,key,f),
        }
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {

        fn process(map:&mut BTreeMap<Hash, Vec<u8>>, key:Hash, data: Vec<u8>) -> Result<()> {
            //insert but give an error if already in
            match map.insert(key, data) {
                None => Ok(()),
                Some(_) => item_already_exists(),
            }
        }

        match class {
            StorageClass::AdtDesc => process(&mut self.0.borrow_mut().adts,key,data),
            StorageClass::FunDesc => process(&mut self.0.borrow_mut().funs,key,data),
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key,data),
            StorageClass::Elem => process(&mut self.0.borrow_mut().elems,key, data),
        }
    }

    //replace a store entry
    fn replace(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {
        fn process(map:&mut BTreeMap<Hash, Vec<u8>>, key:Hash, data: Vec<u8>) -> Result<()> {
            //replace the value but give an error it nothing was there
            match map.insert(key, data) {
                None => item_not_found(),
                Some(_) => Ok(()),
            }
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&mut self.0.borrow_mut().adts,key,data),
            StorageClass::FunDesc => process(&mut self.0.borrow_mut().funs,key,data),
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key,data),
            StorageClass::Elem => process(&mut self.0.borrow_mut().elems,key, data),
        }
    }

    //list all in the store section (this should be used for debug and test only as it would be expensive in production)
    fn list(&self, class: StorageClass) -> Vec<(Hash, Vec<u8>)> {
        fn process(map:&BTreeMap<Hash, Vec<u8>>) -> Vec<(Hash, Vec<u8>)> {
            //clone everithing into mem
            map.iter().map(|(h,v)|(h.clone(),v.clone())).collect()
        }
        //select the right map
        match class {
            StorageClass::AdtDesc => process(&self.0.borrow().adts),
            StorageClass::FunDesc => process(&self.0.borrow().funs),
            StorageClass::Module => process(&self.0.borrow().modules),
            StorageClass::Elem => process(&self.0.borrow().elems),
        }
    }
}
