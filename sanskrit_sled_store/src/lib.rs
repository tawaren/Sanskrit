extern crate sanskrit_common;
extern crate sled;
#[macro_use]
extern crate arrayref;

use sled::Tree;
use sanskrit_common::model::Hash;
use sanskrit_common::store::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;
use std::path::Path;


pub struct SledStore {
    modules:Tree,
    adts:Tree,
    funs:Tree,
    elems:Tree,
}

impl SledStore {
    //creates a multi thread enabled store
    pub fn new(folder:&Path)-> Self{
        let module_p = folder.join("module").with_extension("db");
        let adt_p = folder.join("adt").with_extension("db");
        let fun_p = folder.join("fun").with_extension("db");
        let elem_p = folder.join("elem").with_extension("db");


        SledStore {
            modules: Tree::start_default(module_p).unwrap(),
            adts: Tree::start_default(adt_p).unwrap(),
            funs: Tree::start_default(fun_p).unwrap(),
            elems: Tree::start_default(elem_p).unwrap(),
        }
    }

    pub fn flush(&self) {
        self.modules.flush().unwrap();
        self.adts.flush().unwrap();
        self.funs.flush().unwrap();
        self.elems.flush().unwrap();
    }
}

impl Store for SledStore {
    fn contains(&self, class: StorageClass, key: &Hash) -> bool {
        fn process(map:&Tree, key:&Hash) -> bool {
            map.contains_key(key).unwrap()
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts,key),
            StorageClass::FunDesc => process(&self.funs,key),
            StorageClass::Module => process(&self.modules,key),
            StorageClass::Elem => process(&self.elems,key),
        }
    }

    fn delete(&self, class: StorageClass, key: &[u8; 20])  -> Result<()>  {
        fn process(map: &Tree, key:&Hash) -> Result<()>  {
            match map.del(key).unwrap() {
                None => item_not_found(),
                Some(_) => Ok(()),
            }
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts,key),
            StorageClass::FunDesc => process(&self.funs,key),
            StorageClass::Module => process(&self.modules,key),
            StorageClass::Elem => process(&self.elems,key),
        }
    }


    //Gets a value out and uses P as Parser
    fn get<P, F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        fn process<P>(map:&Tree, key:&Hash, f:F) -> Result<P> {
            //get the store and get the key if available
            match map.get(key).unwrap() {
                None =>  item_not_found(),
                Some(data) => f(&data),
            }
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts,key,f),
            StorageClass::FunDesc => process(&self.funs,key,f),
            StorageClass::Module => process(&self.modules,key,f),
            StorageClass::Elem => process(&self.elems,key,f),
        }
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {

        fn process(map:&Tree, key:Hash, data: Vec<u8>) -> Result<()> {
            match map.set(key, data).unwrap() {
                None => Ok(()),
                Some(_) => item_already_exists(),
            }
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts,key,data),
            StorageClass::FunDesc => process(&self.funs,key,data),
            StorageClass::Module => process(&self.modules,key,data),
            StorageClass::Elem => process(&self.elems,key, data),
        }
    }

    fn replace(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {

        fn process(map:&Tree, key:Hash, data: Vec<u8>) -> Result<()> {
            //if not insert the value under the key
            match map.set(key, data).unwrap() {
                None => item_not_found(),
                Some(_) => Ok(()),
            }
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts,key,data),
            StorageClass::FunDesc => process(&self.funs,key,data),
            StorageClass::Module => process(&self.modules,key,data),
            StorageClass::Elem => process(&self.elems,key, data),
        }
    }



    fn list(&self, class: StorageClass) -> Vec<(Hash, Vec<u8>)> {
        fn process(map:&Tree) -> Vec<(Hash, Vec<u8>)> {
            map.iter().map(|r|{
                let (hash, val) = r.unwrap();
                let hash_data_ref = array_ref!(hash,0,20);
                (hash_data_ref.to_owned(), val.to_owned())
            }).collect()
        }

        match class {
            StorageClass::AdtDesc => process(&self.adts),
            StorageClass::FunDesc => process(&self.funs),
            StorageClass::Module => process(&self.modules),
            StorageClass::Elem => process(&self.elems),
        }
    }
}
