use sanskrit_common::model::Hash;
use core::cell::RefCell;
use core::ops::DerefMut;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;
use alloc::vec::Vec;
use crate::{load, store};
use crate::emit_error;

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
    //delete a store entry
    fn delete(&self, _class: StorageClass, _key: &[u8; 20])  -> Result<()>  {
        //only needed during execution not compilation
        emit_error("Delete not supported");
        unreachable!("Delete not supported")
    }
    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        //we need this scope to make sure lifetime of lock is not to long
        //  else we get in trouble when we call this function recursively
        let size = {
            let mut lock = self.pre_alloc.borrow_mut();
            let workspace: &mut [u8] = &mut lock.deref_mut()[..];
            unsafe{load(class_to_u8(class), key.as_ptr(), workspace.as_mut_ptr(), workspace.len())}
        };
        let res = if size >= 0 {
            Ok(f(&self.pre_alloc.borrow()[0..(size as usize)]))
        } else {
            {
                self.pre_alloc.borrow_mut().resize((-size) as usize, 0);
            }
            self.get(class,key,f)
        };
        res
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {
        if unsafe{store(class_to_u8(class), key.as_ptr(), data.as_ptr(), data.len())}{
            Ok(())
        } else {
            error(||"Store failed")
        }
    }


    fn commit(&self, _class: StorageClass) {
        //we eagerly commit for performance reasons
        //  so this should not be called
        emit_error("Commit not supported");
        unreachable!("Commit not supported")
    }

    fn rollback(&self, _class: StorageClass) {
        //we eagerly commit for performance reasons
        //  so this should not be called
        emit_error("Rollback not supported");
        unreachable!("Rollback not supported")
    }
}
