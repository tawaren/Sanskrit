#![no_std]
#![feature(nll)]

extern crate alloc;
extern crate sanskrit_common;

use alloc::collections::BTreeMap;
use sanskrit_common::model::Hash;
use alloc::vec::Vec;
use core::cell::RefCell;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use core::mem;


#[derive(Clone, Default, Debug)]
struct Container {
    persisted: BTreeMap<Hash, Vec<u8>>,
    pending:BTreeMap<Hash, Option<Vec<u8>>>
}

impl Container {
    pub fn new()-> Self{
        Container{
            persisted: BTreeMap::new(),
            pending: BTreeMap::new()
        }
    }

    pub fn contains_key(&self, key:&Hash) -> bool {
        match self.pending.get(key) {
            None => self.persisted.contains_key(key),
            Some(None) => false,
            Some(Some(_)) => true
        }
    }

    pub fn get(&self, key:&Hash) -> Result<&Vec<u8>> {
        match self.pending.get(key) {
            None => match self.persisted.get(key){
                None => error(||"Value was not in store"),
                Some(res) => Ok(res)
            },
            Some(None) => error(||"Value was not in store"),
            Some(Some(res)) => Ok(res),
        }
    }

    pub fn insert(&mut self, key:Hash, value:Vec<u8>) -> Result<()>  {
        if self.pending.contains_key(&key) || !self.persisted.contains_key(&key) {
            match self.pending.insert(key, Some(value)) {
                None | Some(None)=> Ok(()),
                Some(Some(_)) => error(||"Value was already in store")
            }
        } else  {
            error(||"Value was already in store")
        }
    }


    pub fn remove(&mut self, key:&Hash) -> Result<()> {

        let res = match self.pending.get(key) {
            None => self.persisted.contains_key(key),
            Some(None) => false,
            Some(Some(_)) => true
        };

        if res {
            self.pending.insert(key.clone(), None);
            Ok(())
        } else {
            error(||"Value was not in store")
        }
    }

    pub fn clear(&mut self) {
        self.persisted.clear();
        self.pending.clear();
    }

    pub fn report(&mut self) -> ChangeReport {
        let mut entries_difference = 0;
        let mut bytes_difference = 0;
        for (key, value) in &self.pending {
            match value {
                None => match self.persisted.get(key) {
                    None => {},
                    Some(rem_data) => {
                        entries_difference -= 1;
                        bytes_difference -= rem_data.len() as isize;
                    }
                },
                Some(data) => {
                    entries_difference += 1;
                    bytes_difference += data.len() as isize;
                    match self.persisted.get(key) {
                        None => {},
                        Some(rem_data) => {
                            entries_difference -= 1;
                            bytes_difference -= rem_data.len() as isize;
                        }
                    }
                }
            };
        }

        ChangeReport {
            entries_difference,
            bytes_difference
        }
    }

    pub fn commit(&mut self) {
        let mut res = BTreeMap::new();
        mem::swap(&mut res, &mut self.pending);
        for (key, value) in res {
            match value {
                None => self.persisted.remove(&key),
                Some(data) => self.persisted.insert(key, data)
            };
        }
    }

    pub fn rollback(&mut self){
        self.pending.clear();
    }
}

//A container for the different storage sections
// represented as BTreeMaps (as they work with alloc and HashMap seem not to)
#[derive(Clone, Default, Debug)]
pub struct InnerBTreeMapStore {
    modules:Container,   //All the Modules (Serialized)
    funs:Container,      //All the top level Functions(Serialized)
    descs:Container,     //All the top level Functions(Serialized)
    elems:Container,     //All the Elements (Serialized)
}

//A BTreeMap backed store for development
#[derive(Clone, Default, Debug)]
pub struct BTreeMapStore(RefCell<InnerBTreeMapStore>);

impl BTreeMapStore {
    //creates a single thread enabled store with inner mutability
    pub fn new()-> Self{
        BTreeMapStore(RefCell::new(InnerBTreeMapStore {
            modules: Container::new(),
            funs: Container::new(),
            descs: Container::new(),
            elems: Container::new(),
        }))
    }

    //for testing
    pub fn clear_section(&self, class:StorageClass) {
        fn process(map:&mut Container) {
            map.clear()
        }
        //select the right map
        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
        }

    }
}

impl Store for BTreeMapStore {
    //Checks if the elem is contained
    fn contains(&self, class: StorageClass, key: &Hash) -> bool {
        fn process(map:&Container, key:&Hash) -> bool {
            map.contains_key(key)
        }
        //select the right map
        match class {
            StorageClass::Module => process(&self.0.borrow().modules,key),
            StorageClass::Transaction => process(&self.0.borrow().funs, key),
            StorageClass::Descriptor => process(&self.0.borrow().descs, key),
            StorageClass::EntryValue => process(&self.0.borrow().elems, key),
        }
    }
    //delete a store entry
    fn delete(&self, class: StorageClass, key: &[u8; 20])  -> Result<()>  {
        fn process(map: &mut Container, key:&Hash) -> Result<()>  {
            //remove but gives an error if not their
            map.remove(key)
        }
        //select the right map
        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs, key),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs, key),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems, key),
        }
    }


    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        fn process<P,F:FnOnce(&[u8])-> P>(map: &Container, key:&Hash, f:F) -> Result<P> {
            //get the key if available, else gives an error
            Ok(f(map.get(key)?))
        }
        //select the right map
        match class {
            StorageClass::Module => process(&self.0.borrow().modules,key,f),
            StorageClass::Transaction => process(&self.0.borrow().funs, key, f),
            StorageClass::Descriptor => process(&self.0.borrow().descs, key, f),
            StorageClass::EntryValue => process(&self.0.borrow().elems, key, f),
        }
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {

        fn process(map:&mut Container, key:Hash, data: Vec<u8>) -> Result<()> {
            //insert but gives an error if already in
            map.insert(key, data)
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules,key,data),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs, key, data),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs, key, data),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems, key, data),
        }
    }

    fn report(&self, class: StorageClass) -> ChangeReport {
        fn process(map:&mut Container) -> ChangeReport {
            map.report()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
        }
    }


    fn commit(&self, class: StorageClass) {
        fn process(map:&mut Container) {
            map.commit()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
        }
    }

    fn rollback(&self, class: StorageClass) {
        fn process(map:&mut Container)  {
            map.rollback()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
        }
    }
}
