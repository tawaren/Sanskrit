use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use sanskrit_common::model::Hash;
use sanskrit_common::store::*;
use sanskrit_common::errors::*;


#[derive(Clone, Default, Debug)]
pub struct PreStore(BTreeMap<Hash,Vec<u8>>);

impl PreStore {
    //creates a single thread enabled store with inner mutability
    pub fn new(mods:Vec<Vec<u8>>, pre:Vec<Vec<u8>>)-> (Self, Vec<Hash>){
        let mut map = BTreeMap::new();
        let mut compiles = Vec::with_capacity(mods.len());
        for m in mods {
            let module_hash = store_hash(&[&m]);
            compiles.push(module_hash);
            map.insert(module_hash, m);
        }
        for dep in pre {
            let module_hash = store_hash(&[&dep]);
            map.insert(module_hash, dep);
        }
        (PreStore(map),compiles)
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
