use crate::model::{Hash, hash_from_slice};
use sha2::{Sha256, Digest};

//Hashing Domains to ensure there are no collisions
pub enum HashingDomain {
    Bundle,
    Derive,
    Entry,
    Index
}

pub struct Hasher(Sha256);

impl Hasher {
    pub fn new() -> Self { Hasher(Sha256::new())  }

    pub fn update(&mut self, data:&[u8]) {
        self.0.update(data);
    }

    pub fn finalize(self) -> Hash {
        //calc the Hash
        let hash = self.0.finalize();
        //generate a array to the hash
        hash_from_slice(&hash[..])
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

