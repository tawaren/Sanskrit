use errors::*;
use encoding::*;
use alloc::prelude::*;
use model::Hash;
use hashing::*;

//Trait representing a store
//Allows it to be flexible from Temporary in Memeory, over stateless in Memeory to persistent
pub trait Store {
    //Check if something is their
    fn contains(&self, class:StorageClass, key: &Hash) -> bool;
    //Check if something is their
    fn delete(&self, class:StorageClass, key: &Hash) -> Result<()>;
    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8]) -> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P>;
    //Stores a value in the store (reqiures it is empty)
    fn set(&self, class:StorageClass, key:Hash, data:Vec<u8>) -> Result<()> ;
    //Stores a value in the store replacing the existing
    fn replace(&self, class:StorageClass, key:Hash, data:Vec<u8>) -> Result<()> ;
    //Lists all elems from that category
    //just for debug and test, not suitable for rest as it copies the whole store to memory
    //fn list(&self, class:StorageClass) -> Vec<(Hash, Vec<u8>)>;

    //helper
    fn parsed_get<'a, P:Parsable<'a>, A: ParserAllocator>(&self, class:StorageClass, key: &Hash, max_dept:usize, alloc:&'a A) -> Result<P>{
       self.get(class, key,|d| Parser::parse_fully::<P,A>(d, max_dept, alloc))?
    }

    fn serialized_set<S:Serializable,>(&self, class:StorageClass, key:Hash, max_dept:usize, data:&S) -> Result<()>{
        self.set(class,key, Serializer::serialize_fully(data,max_dept)?)
    }

    fn serialized_replace<S:Serializable,>(&self, class:StorageClass, key:Hash, max_dept:usize, data:&S) -> Result<()>{
        self.replace(class,key, Serializer::serialize_fully(data,max_dept)?)
    }
}

//enum pointing to different sections in the store
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum StorageClass{
    AdtDesc, FunDesc, Module, Elem
}

//Helper to calc the key for a storage slot
pub fn store_hash(data:&[&[u8]]) -> Hash {
    //Make a 20 byte digest hascher
    let mut context = HashingDomain::Code.get_domain_hasher();
    //push the data into it
    for d in data {
        context.update(*d);
    }
    //calc the Hash
    context.finalize()
}