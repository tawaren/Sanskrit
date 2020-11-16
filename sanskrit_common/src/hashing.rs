use model::{Hash, SlicePtr, hash_from_slice};
use errors::*;
use arena::VirtualHeapArena;

//Hashing Domains to ensure there are no collisions
pub enum HashingDomain {
    Bundle,
    Derive,
    Entry,
    Index
}

pub struct Hasher(blake3::Hasher);

impl Hasher {
    pub fn new() -> Self { Hasher(blake3::Hasher::new())  }

    pub fn update(&mut self, data:&[u8]) {
        self.0.update(data);
    }

    pub fn alloc_finalize<'a>(self, alloc:&'a VirtualHeapArena) ->  Result<SlicePtr<'a,u8>>{
        //calc the Hash
        let hash = self.0.finalize();
        //generate a array to the hash
        let hash_data_ref = array_ref!(hash.as_bytes(),0,20);
        //allocate on the heap
        alloc.copy_alloc_slice(hash_data_ref)
    }

    pub fn finalize(self) -> Hash {
        //calc the Hash
        let hash = self.0.finalize();
        //generate a array to the hash
        hash_from_slice(hash.as_bytes())
    }
}

impl HashingDomain {

    pub fn get_domain_code(&self) -> u8 {
        match *self {
            HashingDomain::Derive => 0,
            HashingDomain::Bundle => 1,
            HashingDomain::Entry => 2,
            HashingDomain::Index => 3
        }
    }

    pub fn get_domain_hasher(&self) -> Hasher {
        let mut context = Hasher::new();
        context.update(&[self.get_domain_code()]);
        context
    }

    //Helper to calc the input hash
    pub fn hash(&self, data:&[u8]) -> Hash {
        //Make a 20 byte digest hascher
        let mut context = self.get_domain_hasher();
        //push the data into it
        context.update(data);
        //calc the Hash
        context.finalize()
    }

}

