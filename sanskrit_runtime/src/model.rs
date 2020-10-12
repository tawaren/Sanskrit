use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;

#[derive(Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct DeployTransaction<#[AllocLifetime] 'c> {
    pub typ:DeployType,
    pub data: SlicePtr<'c, u8>,
    //Accounting
    pub max_load_bytes:u32,
    pub max_store_bytes:u32,
    pub max_process_bytes:u32,
    pub max_stack_elems:u32,
    pub max_block_nesting:u32,
    //Limits Compiling (only needed for transactions)
    pub max_contained_functions:u32,          //todo: shall we hardcode these
    pub max_compile_block_nesting:u32,        //todo: shall we hardcode these
}


#[derive(Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct TransactionBundleCore<#[AllocLifetime] 'c> {
    #[ByteSize]
    pub byte_size:Option<usize>,
    //freely usable by the sender
    //can be used to change hash or add meta information
    pub meta:SlicePtr<'c, u8>,
    //The earliest block where this is allowed to be included
    // The latest block is defined by earliest_block + TXT_BLOCK_WINDOW
    // This simplifies TXT double inclusion check, as we only need to check TXT_BLOCK_WINDOW blocks back
    //  We check this to ensure TXT_HASHes are unique
    pub earliest_block:u64,
    //maximum size accepted for params
    pub param_heap_limit: u16,
    //Static execution costs
    pub transaction_storage_heap: u16,
    pub stack_elem_limit:u16,
    pub stack_frame_limit:u16,
    pub runtime_heap_limit:u16,
    //Gas cost
    pub essential_gas_cost:u64, //Gas cost a miner must spend before he knows if the transaction is valid and pays him
    pub total_gas_cost:u64,
    //Transaction Sections
    pub sections: SlicePtr<'c, BundleSection<'c>>,
    //Constants
    pub descriptors: SlicePtr<'c, Hash>,
    //ids for params loaded from the blockchain
    pub stored: SlicePtr<'c, Hash>,
    //params passed in from the outside
    pub literal: SlicePtr<'c, SlicePtr<'c,u8>>,
    //witness limit (prevents miner to add witnesses to earn more)
    //Todo: These is only needed in Witness Mode which we will abstract away not doing in the beginning
    //pub witness_bytes_limit:u32, //todo: maybe just u16?
}

//todo: an intermidiary passing option
//      place on parameter heap
//      have an Vec (with bound)
//       it contains: Type|TypeRef for managing them between borders
//A set of transactions
#[derive(Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct TransactionBundle<#[AllocLifetime] 'c> {
    #[ByteSize]
    pub byte_size:Option<usize>,
    //everything that is part of the hash
    pub core: TransactionBundleCore<'c>,
    //witnesses
    pub witness: SlicePtr<'c, SlicePtr<'c,u8>>,                     //witnesses are ignored in the Hash
    //todo: This is only in Witness mode (which we may not use at the start)
    //      Note: These changes the gas formula??
    //pub store_witness: SlicePtr<'c, Option<SlicePtr<'c,u8>>>,       //witnesses are ignored in the Hash

}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Parsable, Serializable, VirtualSize)]
pub enum SectionType {
    Essential,
    Custom
}

#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum DeployType {
    Module,
    Transaction
}

//A section of transactions
#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub struct BundleSection<#[AllocLifetime] 'c> {
    //Section type
    pub typ:SectionType,
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