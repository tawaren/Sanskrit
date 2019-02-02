use blake2_rfc::blake2b::{Blake2b};
use byteorder::{LittleEndian, ByteOrder};

//Hashing Domains to ensure there are no collisions
pub enum HashingDomain {
    Unique,
    Singleton,
    Transaction,
    Id,
    Derive,
    Object,
    Account
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
        }
    }

    pub fn get_domain_hasher(&self, input_len:i32) -> Blake2b {
        let mut context = Blake2b::new(20);
        //prepare the counter
        let mut input = [0u8; 4];
        LittleEndian::write_i32(&mut input, input_len);
        context.update(&[self.get_domain_code()]);
        context.update(&input);
        context
    }
}