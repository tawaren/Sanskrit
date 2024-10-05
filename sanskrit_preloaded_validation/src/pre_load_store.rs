use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use sanskrit_common::model::Hash;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;


#[derive(Clone, Default, Debug)]
pub struct PreStore(BTreeMap<Hash,Vec<u8>>);

impl PreStore {
    //creates a single thread enabled store with inner mutability
    pub fn new()-> Self{
        PreStore(BTreeMap::new())
    }

    pub fn add_module(self:&mut Self, module:Vec<u8>) -> Hash {
        let module_hash = store_hash(&[&module]);
        self.0.insert(module_hash, module);
        module_hash
    }
}

impl Store for PreStore {
    //delete a store entry
    fn delete(&self, class: StorageClass, key: &[u8; 20]) -> Result<()>  {
        error(||"No delete supported")
    }

    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8])-> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P> {
        //select the right map
        match class {
            StorageClass::Module => match self.0.get(key){
                None => error(||"module not found"),
                Some(module) => Ok(f(&module))
            },
            _  => error(||"only modules are supported"),
        }
    }

    //Stores a value in the store
    fn set(&self, class:StorageClass, key:Hash, data: Vec<u8>) -> Result<()> {
        //later we may check that this is done
        Ok(())
    }

    fn commit(&self, class: StorageClass) { }
    fn rollback(&self, class: StorageClass) {}
}
