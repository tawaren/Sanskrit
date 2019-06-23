use sanskrit_common::model::*;
use sanskrit_common::capabilities::CapSet;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;
use sanskrit_interpreter::model::*;


//A transaction
#[derive(Copy, Clone, Parsable, Serializable, VirtualSize)]
pub struct Transaction<#[AllocLifetime] 'c> {
    //Config:
    pub start_block_no: u64,
    //the earliest block to include it
    //Todo: Memory Claim
    //Todo: Stack Depth Claim / Stack Heap Claim
    //Todo: Desc Buffer Claim
    //Todo: Entry Cache Claim
    //Consts:
    pub signers: SlicePtr<'c, [u8; 32]>, //todo: Not needed if we can recover from sig
    pub imports: SlicePtr<'c, Hash>,
    pub new_types:u8,
    pub code: SlicePtr<'c, Ptr<'c, ScriptCode<'c>>>,
    pub signatures: SlicePtr<'c, [u8; 64]>,
    pub witness: SlicePtr<'c, SlicePtr<'c,u8>>,
}

/*
//The Native Adt types
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum NativeAdtType {
    Tuple(u8),
    Alternative(u8),
    Bool,
}
*/

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct ImpRef(pub u8);

//A reference to identify an Adt descriptor
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct AdtRef {
    pub module:ImpRef,
    pub offset:u8
}



//A reference to identify an Adfunctiont descriptor
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct FuncRef{
    pub module:ImpRef,
    pub offset:u8
}


#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeApplyRef<#[AllocLifetime] 'c> {
    Account(u8),                //Count as Priviledged (can call protected)
    RemoteAccount(ImpRef),
    NewType(u8),                //Count as Priviledged (can call protected)
    RemoteNewType(ImpRef,u8),
    TypeOf(ValueRef),
    ArgTypeOf(ValueRef, SlicePtr<'c, u8>),
    Module(ImpRef, u8,  SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>),
    Image(Ptr<'c,TypeApplyRef<'c>>),
}

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum ScriptCode<#[AllocLifetime] 'c> {
    Pack(AdtRef, SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>, Tag, SlicePtr<'c,ValueRef>),           //Packs an adt
    BorrowPack(AdtRef, SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>, Tag, SlicePtr<'c,ValueRef>),     //Generate an adt by borrowing the fields
    //todo: ImagePack
    Unpack(AdtRef, Tag, ValueRef),                                                              //Unpack an adt
    BorrowUnpack(AdtRef, Tag, ValueRef),                                                        //Borrow the fields of an adt
    //todo: ImageUnpack
    Invoke(FuncRef, SlicePtr<'c,Ptr<'c,TypeApplyRef<'c>>>, SlicePtr<'c,ValueRef>),              //Call a function
    Lit(SlicePtr<'c, u8>, LitDesc, AdtRef),                                                             //Generate a literal
    Wit(u8, LitDesc, AdtRef),                                                                           //Generate a literal from a Witness
    Copy(ValueRef),                                                                             //Copy a stack value
    Fetch(ValueRef),                                                                            //Move a stack value
    BorrowFetch(ValueRef), //Borrow a stack value
    //todo: ImageFetch
    //todo: FlattenImage
    Free(ValueRef),                                                                             //Free a borrowed stack value
    Drop(ValueRef),                                                                             //Drop a stack value
    Load(ValueRef),                                                                             //Load a value from the store
    BorrowLoad(ValueRef),                                                                       //Borrow a value from the store
    Store(ValueRef),                                                                            //Save a value to the store
    //todo: Should we have a Derive Here
}

//todo: MeasuredRuntimeType
//  (nodes, leaves, empty_nodes, RuntimeType) //the former is used to calc gas cost of comparing
#[derive(Clone, Copy, Eq, PartialEq, Debug, Parsable, Serializable, VirtualSize)]
pub enum RuntimeType<#[AllocLifetime] 'a> {
    Custom {
        caps: CapSet,
        module: Hash,
        offset: u8,
        applies: SlicePtr<'a, Ptr<'a, RuntimeType<'a>>>
    },

    Image {
        typ:  Ptr<'a, RuntimeType<'a>>
    },

    NewType {
        txt: Hash,
        offset: u8,
    },

    AccountType {
        address: Hash,
    },
}

//an element in the backing store
#[derive(Clone, Copy, Eq, PartialEq, Debug, Parsable, Serializable)]
pub struct StoreElem<#[AllocLifetime] 'a> {
    pub val: Ptr<'a, Object<'a>>,
    pub typ: Ptr<'a, RuntimeType<'a>>
}