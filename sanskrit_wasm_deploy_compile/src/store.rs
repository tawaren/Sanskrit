use sanskrit_common::model::Hash;
use core::cell::RefCell;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use core::mem;
use std::collections::BTreeMap;
use crate::{contains, load, store};
use std::borrow::Borrow;

//A BTreeMap backed store for development
#[derive(Clone, Default, Debug)]
pub struct ExternalStore{
    pre_alloc:RefCell<Vec<u8>>
}

impl ExternalStore {
    //creates a single thread enabled store with inner mutability
    pub fn new(pre_alloc:usize)-> Self{
        let mut data = Vec::with_capacity(pre_alloc);
        data.resize(pre_alloc, 0);
        ExternalStore{
            pre_alloc:RefCell::new(data)
        }
    }
}

fn class_to_u8(class:StorageClass) -> u8 {
    class as u8
}

impl Store for ExternalStore {
    //Checks if the elem is contained
    fn contains(&self, class: StorageClass, key: &Hash) -> bool {
        unsafe{contains(class_to_u8(class), key.as_ptr())}
    }

    //delete a store entry
    fn delete(&self, class: StorageClass, key: &[u8; 20])  -> Result<()>  {
        //only needed during execution not compilation
        unreachable!()
    }


    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        let size = unsafe{load(class_to_u8(class), key.as_ptr(), self.pre_alloc.borrow_mut().as_mut_ptr(), self.pre_alloc.borrow().len())};
        if size >= 0 {
            Ok(f(&self.pre_alloc.borrow()[0..(size as usize)]))
        } else {
            self.pre_alloc.borrow_mut().resize((-size) as usize, 0);
            self.get(class,key,f)
        }
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {
        if unsafe{store(class_to_u8(class), key.as_ptr(), data.as_ptr(), data.len())}{
            Ok(())
        } else {
            error(||"Store failed")
        }
    }


    fn commit(&self, class: StorageClass) {
        //we eagerly commit for performance reasons
        //  so this should not be called
        unreachable!()
    }

    fn rollback(&self, class: StorageClass) {
        //we eagerly commit for performance reasons
        //  so this should not be called
        unreachable!()
    }
}
