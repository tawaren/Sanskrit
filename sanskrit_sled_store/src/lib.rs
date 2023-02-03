extern crate sanskrit_common;
extern crate sled;
extern crate arrayref;
extern crate core;

use sled::Db;
use sanskrit_common::model::Hash;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, BTreeSet};
use core::mem;
use std::cell::RefCell;

#[derive(Clone, Debug)]
struct Container {
    persisted:Db,
    pending:BTreeMap<Hash, Option<Vec<u8>>>,
    auto_flush:bool
}

impl Container {
    pub fn new(path:PathBuf, auto_flush:bool)-> Self{
        Container{
            persisted: sled::open(path).unwrap(),
            pending: BTreeMap::new(),
            auto_flush
        }
    }

    pub fn contains_key(&self, key:&Hash) -> bool {
        match self.pending.get(key) {
            None => self.persisted.contains_key(key).unwrap(),
            Some(None) => false,
            Some(Some(_)) => true
        }
    }

    pub fn get<P,F:FnOnce(&[u8])-> P>(&self, key:&Hash, f:F) -> Result<P> {
        match self.pending.get(key) {
            None => match self.persisted.get(key).unwrap(){
                None => error(||"Value was not in store"),
                Some(res) => Ok(f(&res))
            },
            Some(None) => error(||"Value was not in store"),
            Some(Some(res)) => Ok(f(res)),
        }
    }

    pub fn insert(&mut self, key:Hash, value:Vec<u8>) -> Result<()>  {
        if self.pending.contains_key(&key) || !self.persisted.contains_key(&key).unwrap() {
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
            None => self.persisted.contains_key(key).unwrap(),
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

    pub fn commit(&mut self) {
        let mut res = BTreeMap::new();
        mem::swap(&mut res, &mut self.pending);
        for (key, value) in res {
            match value {
                None => self.persisted.remove(&key).unwrap(),
                Some(data) => self.persisted.insert(key, data).unwrap()
            };
        }
        if self.auto_flush {
            self.persisted.flush().unwrap();
        }
    }

    pub fn rollback(&mut self){
        self.pending.clear();
    }

    pub fn flush(&mut self) {
        self.persisted.flush().unwrap();
    }
}

pub struct InnerSledStore {
    hashs: Container,
    modules: Container,
    funs: Container,
    descs: Container,
    elems: Container,
}

pub struct SledStore(RefCell<InnerSledStore>);

impl SledStore {
    //creates a multi thread enabled store
    pub fn new(folder:&Path, auto_flush:BTreeSet<StorageClass>)-> Self{
        let hash_p = folder.join("hash").with_extension("db");
        let module_p = folder.join("module").with_extension("db");
        let fun_p = folder.join("fun").with_extension("db");

        let desc_p = folder.join("desc").with_extension("db");
        let elem_p = folder.join("elem").with_extension("db");


        SledStore(RefCell::new(InnerSledStore {
            hashs: Container::new(hash_p, auto_flush.contains( &StorageClass::EntryHash)),
            modules: Container::new(module_p, auto_flush.contains( &StorageClass::Module)),
            funs: Container::new(fun_p, auto_flush.contains( &StorageClass::Transaction)),
            descs: Container::new(desc_p, auto_flush.contains( &StorageClass::Descriptor)),
            elems: Container::new(elem_p, auto_flush.contains( &StorageClass::EntryValue)),
        }))
    }

    pub fn flush(&self, class: StorageClass) {
        fn process(map:&mut Container) {
            map.flush()
        }

        match class {
            StorageClass::Module => process(&mut self.0.borrow_mut().modules),
            StorageClass::Transaction => process(&mut self.0.borrow_mut().funs),
            StorageClass::Descriptor => process(&mut self.0.borrow_mut().descs),
            StorageClass::EntryValue => process(&mut self.0.borrow_mut().elems),
            StorageClass::EntryHash =>  process(&mut self.0.borrow_mut().hashs),
        }
    }
}


impl Store for SledStore {
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
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs, key),
        }
    }


    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        fn process<P,F:FnOnce(&[u8])-> P>(map: &Container, key:&Hash, f:F) -> Result<P> {
            //get the key if available, else gives an error
            map.get(key,f)
        }
        //select the right map
        match class {
            StorageClass::Module => process(&self.0.borrow().modules,key,f),
            StorageClass::Transaction => process(&self.0.borrow().funs, key, f),
            StorageClass::Descriptor => process(&self.0.borrow().descs, key, f),
            StorageClass::EntryValue => process(&self.0.borrow().elems, key, f),
            StorageClass::EntryHash => process(&self.0.borrow().hashs, key, f),
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
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs, key, data),
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
            StorageClass::EntryHash =>  process(&mut self.0.borrow_mut().hashs),
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
            StorageClass::EntryHash => process(&mut self.0.borrow_mut().hashs),
        }
    }


    /*
    fn list(&self, class: StorageClass) -> Vec<(Hash, Vec<u8>)> {
        fn process(map:&Tree) -> Vec<(Hash, Vec<u8>)> {
            map.iter().map(|r|{
                let (hash, val) = r.unwrap();
                let hash_data_ref = array_ref!(hash,0,20);
                (hash_data_ref.to_owned(), val.to_owned())
            }).collect()
        }

        match class {
            StorageClass::AdtDesc => process(&self.adt_descs),
            StorageClass::FunDesc => process(&self.fun_descs),
            StorageClass::Module => process(&self.0.borrow().modules),
            StorageClass::Function => process(&self.0.borrow().funs),
            StorageClass::Elem => process(&self.0.borrow().elems),
        }
    }
    */
}
