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
    //does this deploy a module or transaction?
    //if so the normal part pays for it
    pub deploy: Option<DeployTransaction<'c>>,
    //maximum size accepted for params
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
    pub stored: SlicePtr<'c, Hash>,
    //params passed in from the outside
    pub literal: SlicePtr<'c, SlicePtr<'c,u8>>,
    //witness limit (prevents miner to add witnesses to earn more)
    pub witness_bytes_limit:u32, //todo: maybe just u16?
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
    //everithing that is part of the hash
    pub core: TransactionBundleCore<'c>,
    //witnesses
    pub witness: SlicePtr<'c, SlicePtr<'c,u8>>,                     //witnesses are ignored in the Hash
    //Todo: Remove the None case <-- it is very usefull for testing keep as long as possible
    //Todo: Make Witness adaption programm -- store last X values
    //       Algo: we go over and check and upodate them if we have never
    pub store_witness: SlicePtr<'c, Option<SlicePtr<'c,u8>>>,       //witnesses are ignored in the Hash

}

#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum SectionType {
    Payment,
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