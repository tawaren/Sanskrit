use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;
use sanskrit_interpreter::model::*;
use alloc::vec::Vec;

//A set of transactions
#[derive(Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct TransactionBundle<#[AllocLifetime] 'c> {
    //Todo: Add o witness
    pub param_heap_limit: u16,
    //Static execution costs
    pub transaction_storage_heap: u16,
    pub stack_elem_limit:u16,
    pub stack_frame_limit:u16,
    pub runtime_heap_limit:u16,
    //Transaction Sections
    pub sections: SlicePtr<'c, BundleSection<'c>>,
    //Constants
    pub descriptors: SlicePtr<'c, Hash>,
    //ids for params loaded from the blockchain
    pub stored: SlicePtr<'c, SlicePtr<'c,u8>>,
    //params passed in from the outside
    pub literal: SlicePtr<'c, SlicePtr<'c,u8>>,
    //witnesses
    pub witness: SlicePtr<'c, SlicePtr<'c,u8>>,                 //witnesses are ignored in the Hash
    pub store_witness: SlicePtr<'c, Option<SlicePtr<'c,u8>>>,   //witnesses are ignored in the Hash

}

#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum SectionType {
    Payment,
    Custom
}

//A section of transactions
#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub struct BundleSection<#[AllocLifetime] 'c> {
    //Section type
    pub typ:SectionType,
    //Storage costs         -- we later could optimize: load -> delete -> store (as in reality this triggers 1 Load & 1 Write and not 2 writes)
    pub entries_loaded:u32,
    pub entries_created:u32,
    pub entries_deleted:u32,
    //Execution Cost
    pub gas_limit: u64,
    //Transactions
    pub txts: SlicePtr<'c, Transaction<'c>>,
}


//A transaction
#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub struct Transaction<#[AllocLifetime] 'c> {
    //transaction type
    pub txt_desc: u16,
    //parameter source & fetch mode
    pub params: SlicePtr<'c, ParamRef>,
    //parameter source & fetch mode
    pub returns: SlicePtr<'c, RetType>,
}


#[derive(Copy, Eq, PartialEq, Clone, Parsable, Serializable, VirtualSize, Debug)]
pub enum ParamMode {
    Copy,
    Borrow,
    Consume
}

#[derive(Copy, Eq, PartialEq, Clone, Parsable, Serializable, VirtualSize, Debug)]
pub enum ParamRef {
    Load(ParamMode, u16),
    //Fetch(ParamMode, u16),
    Literal(u16),
    Witness(u16),
    Provided
}

#[derive(Copy, Eq, PartialEq, Clone, Parsable, Serializable, VirtualSize, Debug)]
pub enum RetType {
    Store,
    //Put(u16),
    Drop,
    Log
}