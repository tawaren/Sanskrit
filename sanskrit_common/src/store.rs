use errors::*;
use encoding::*;
use alloc::vec::Vec;

use model::Hash;
use hashing::*;

pub struct ChangeReport {
    pub entries_difference: isize,
    pub bytes_difference: isize,
}

//Trait representing a store
//Allows it to be flexible from Temporary in Memeory, over stateless in Memeory to persistent
pub trait Store {
    //Check if something is their
    fn contains(&self, class:StorageClass, key: &Hash) -> bool;
    //Check if something is their
    fn delete(&self, class:StorageClass, key: &Hash) -> Result<()>;
    //Gets a value out and uses P as Parser
    fn get<P,F:FnOnce(&[u8]) -> P>(&self, class:StorageClass, key: &Hash, f:F) -> Result<P>;
    //Stores a value in the store (reqiures that it is empty)
    fn set(&self, class:StorageClass, key:Hash, data:Vec<u8>) -> Result<()> ;
    // reports the pending elements
    fn report(&self, class:StorageClass) -> ChangeReport;
    //commits accumulated changes
    fn commit(&self, class:StorageClass);
    //reverts accumulated changes;
    fn rollback(&self, class:StorageClass);

    //helper
    fn parsed_get<'a, P:Parsable<'a>, A: ParserAllocator>(&self, class:StorageClass, key: &Hash, max_dept:usize, alloc:&'a A) -> Result<P>{
       self.get(class, key,|d| Parser::parse_fully::<P,A>(d, max_dept, alloc))?
    }

    fn serialized_set<S:Serializable,>(&self, class:StorageClass, key:Hash, max_dept:usize, data:&S) -> Result<()>{
        self.set(class,key, Serializer::serialize_fully(data,max_dept)?)
    }

}

//enum pointing to different sections in the store
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum StorageClass{
    Module,
    Transaction,
    Descriptor,
    EntryHash,  // hash(type||value)
    EntryValue //Value will only be needed by state providers
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