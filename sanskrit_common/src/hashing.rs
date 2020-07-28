use blake2_rfc::blake2b::{Blake2b};
use model::{Hash, SlicePtr};
use errors::*;
use arena::VirtualHeapArena;

//Hashing Domains to ensure there are no collisions
pub enum HashingDomain {
    Unique,
    Singleton,
    Transaction,
    Bundle,
    Id,
    Derive,
    Object,
    Account,
    Code,
    Entry
}

pub struct Hasher(Blake2b);

impl Hasher {
    pub fn new() -> Self {
        Hasher(Blake2b::new(20))
    }

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
        array_ref!(hash.as_bytes(),0,20).clone()
    }
}

impl HashingDomain {

    pub fn get_domain_code(&self) -> u8 {
        match *self {
            HashingDomain::Unique => 0,
            HashingDomain::Singleton => 1,
            HashingDomain::Transaction => 2,
            HashingDomain::Id => 3,
            HashingDomain::Derive => 4,
            HashingDomain::Object => 5,
            HashingDomain::Account => 6,
            HashingDomain::Code => 7,
            HashingDomain::Bundle => 8,
            HashingDomain::Entry => 9,
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

