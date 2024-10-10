use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use crate::errors::*;
use crate::encoding::*;
use crate::model::Hash;
use crate::hashing::*;
use crate::utils::Crc;

//Trait representing a store
//Allows it to be flexible from Temporary in Memory, over stateless in Memory to persistent
pub trait Supplier<P> {
    //guarantees: unique_get(x)?.as_ref() == unique_get(y)?.as_ref() if x.eq(y)
    fn unique_get(&self, key: &Hash) -> Result<Crc<P>>;
}

pub struct ParsedSupplier<P> {
    supply:BTreeMap<Hash, Crc<P>>
}

impl<P:Parsable> ParsedSupplier<P> {
    pub fn new() -> Self {
        ParsedSupplier {
            supply: BTreeMap::new()
        }
    }

    pub fn add(&mut self, data:&[u8]) -> Result<Hash> {
        let key = store_hash(&[data]);
        if !self.supply.contains_key(&key) {
            let parsed: P = Parser::parse_fully(data)?;
            let val:Crc<P> = Crc{elem:Rc::new(parsed)};
            self.supply.insert(key.clone(), val);
            Ok(key)
        } else {
            Ok(key)
        }
    }
}

impl<P> Supplier<P> for ParsedSupplier<P>  {
    fn unique_get(&self, key: &Hash) -> Result<Crc<P>> {
        match self.supply.get(key) {
            None => error(||"Required module is missing"),
            Some(content) => Ok(content.clone())
        }
    }
}

//Helper to calc the key for a storage slot
pub fn store_hash(data:&[&[u8]]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = Hasher::new();
    //push the data into it
    for d in data {
        context.update(*d);
    }
    //calc the Hash
    context.finalize()
}
