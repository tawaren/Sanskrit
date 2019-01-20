use sanskrit_common::model::*;
use sanskrit_common::capabilities::CapSet;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;


//A transaction
#[derive(Copy, Clone, Parsable, Serializable, VirtualSize)]
pub struct Transaction<#[AllocLifetime] 'c> {
    pub start_block_no: u64,
    //the earliest block to include it
    //Todo: Memory Claim
    //Todo: Stack Depth Claim / Stack Heap Claim
    //Todo: Desc Buffer Claim
    //Todo: Entry Cache Claim
    pub signers: SlicePtr<'c, [u8; 32]>,
    pub code: SlicePtr<'c, Ptr<'c, ScriptCode<'c>>>,
    pub signatures: SlicePtr<'c, [u8; 64]>,
}

//bool == is is_phantom
#[derive(Copy, Clone, Eq, PartialEq, Debug, Parsable, Serializable, VirtualSize)]
pub struct TypeTypeParam(pub bool, pub CapSet);

//an identifier of an adt
#[derive(Copy, Clone, Debug, Parsable, Serializable)]
pub enum AdtId {
    Custom(Hash,u8),        //Hash is module Hash
    Native(NativeType)
}

#[derive(Copy, Clone, Debug, Parsable, Serializable)]
pub struct AdtDescriptor<#[AllocLifetime] 'b> {
    pub generics:SlicePtr<'b, TypeTypeParam>,
    pub constructors:SlicePtr<'b,SlicePtr<'b,TypeBuilder<'b>>>,
    pub base_caps:CapSet,                   //the allowed capabilities
    pub id: AdtId,
}

//todo: make named struct
//1: bool == is_protected
//2: bool == is_phantom
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct FunTypeParam{
    pub is_protected:bool,
    pub is_phantom:bool,
    pub caps:CapSet
}

//bool = is_consume
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct Param<#[AllocLifetime] 'b>(pub bool, pub Ptr<'b,TypeBuilder<'b>>);

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct Return<#[AllocLifetime] 'b>(pub Ptr<'b,TypeBuilder<'b>>, pub SlicePtr<'b,ValueRef>);

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct FunctionDescriptor<#[AllocLifetime] 'b> {
    pub gas_cost:u32,
    pub max_stack:u16,
    pub max_frames:u16,
    //todo: later do mem
    pub generics:SlicePtr<'b,FunTypeParam>,
    pub params:SlicePtr<'b,Param<'b>>,
    pub returns:SlicePtr<'b,Return<'b>>,
    pub functions:SlicePtr<'b,Ptr<'b, Exp<'b>>>,        //multiple functions (calls are embeded)
}

//The Native Adt types
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum NativeAdtType {
    Tuple(u8),
    Alternative(u8),
    Bool,
}

//A reference to identify an Adt descriptor
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum AdtRef {
    Dynamic(ValueRef),
    Ref(Hash),
    Native(NativeAdtType)
}

//A reference to identify an Adfunctiont descriptor
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum FuncRef{
    Dynamic(ValueRef),
    Ref(Hash),
}

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeApplyRef<#[AllocLifetime] 'c> {
    Account(u8),            //Count as Priviledged (can call protected)
    RemoteAccount(Hash),
    NewType(u8),            //Count as Priviledged (can call protected)
    RemoteNewType(Hash,u8),
    Value(ValueRef),
    Native(NativeType, SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>),
    Module(Hash,  SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>),

}

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum ScriptCode<#[AllocLifetime] 'c> {
    Pack(AdtRef, SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>, Tag, SlicePtr<'c,ValueRef>),           //Packs an adt
    BorrowPack(AdtRef, SlicePtr<'c, Ptr<'c,TypeApplyRef<'c>>>, Tag, SlicePtr<'c,ValueRef>),     //Generate an adt by borrowing the fields
    Unpack(AdtRef, Tag, ValueRef),                                                              //Unpack an adt
    BorrowUnpack(AdtRef, Tag, ValueRef),                                                        //Borrow the fields of an adt
    Invoke(FuncRef, SlicePtr<'c,Ptr<'c,TypeApplyRef<'c>>>, SlicePtr<'c,ValueRef>),              //Call a function
    Lit(SlicePtr<'c, u8>, LitDesc),                                                             //Generate a literal
    Copy(ValueRef),                                                                             //Copy a stack value
    Fetch(ValueRef),                                                                            //Move a stack value
    BorrowFetch(ValueRef),                                                                      //Borrow a stack value
    Free(ValueRef),                                                                             //Free a borrowed stack value
    Drop(ValueRef),                                                                             //Drop a stack value
    Load(ValueRef),                                                                             //Load a value from the store
    BorrowLoad(ValueRef),                                                                       //Borrow a value from the store
    Store(ValueRef),                                                                            //Save a value to the store
    NewType,                                                                                    //Generte a new type and a singleton value for it
    //todo: Should we have a Derive Here
}

//A type to represent an error in the interpreter
#[derive(Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum Error{
    Native(NativeError),
    Custom(u16)             //is shortened and mapped to prevent having Module Hashes everywhere
}

//A Block
#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum Exp<#[AllocLifetime] 'b> {
    Ret(SlicePtr<'b, OpCode<'b>>,SlicePtr<'b,ValueRef>),
    Throw(Error),
}

//Description for a literal type
#[derive(Copy, Clone, Eq, PartialEq, Debug, Parsable, Serializable, VirtualSize)]
#[repr(u8)]
pub enum LitDesc {
    Ref,
    Data,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    I128,
    U128,
}

//Represents a function that can be called
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
pub enum FunDesc {
    Native(Operand),
    Custom(u16),        //index of the embeded function (eliminates Module Hashes)
}

//All the Available Native Funcitions
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
pub enum Operand {
    And,        //Deploys a logical and on bools or bitwise on ints
    Or,         //Deploys a logical or on bools or bitwise on ints
    Xor,        //Deploys a logical xor on bools or bitwise on ints
    Not,        //Deploys a logical not on bools or bitwise on ints
    ToU(u8),    //cast to an Unsigned Integer with u8 Bytes
    ToI(u8),    //cast to an Signed Integer with u8 Bytes
    Add,        //Does an arithmetic addition of two ints (throws on under or overflow)
    Sub,        //Does an arithmetic subtraction of two ints (throws on under or overflow)
    Mul,        //Does an arithmetic multiplication of two ints (throws on under or overflow)
    Div,        //Does an arithmetic dividation of two ints (throws on a division by zero)
    Eq,         //Compares two values for equality
    Hash,       //Calculates the hash of a value
    PlainHash,  //Calculates a plain hash for a data input (not structurally encoded)
    Lt,         //Compares two values to decide if one is less than the other
    Gt,         //Compares two values to decide if one is greater than the other
    Lte,        //Compares two values to decide if one is less than or equal the other
    Gte,        //Compares two values to decide if one is greater or equal than the other
    ToData,     //Transforms Integers & Uniques to data
    Concat,     //Concats two data values
    SetBit,     //sets a bit in a data value
    GetBit,     //queries a bit from a data value
    GenUnique,  //generates a new unique value (needs context for that)
    FullHash,   //gets the full hash of the current transactoion (needs context for that)
    TxTHash,    //gets the transaction hash (no witnesses) of the current transactoion (needs context for that)
    CodeHash,   //gets the hash of the currents transactions code  (needs context for that)
    BlockNo,    //gets the blockno in which the transaction is included
    GenIndex,   //generates a new storage index fro data or uniques
    Derive,     //derives a new index or referenz from two others
    //Gas Testing Operands
    Id,         //Makes a Copy of the input (this is for testing) -- Establishes a Baseline


}

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
//todo: lift natives on this level -- else we do 2 additional unecessary jumps
// todo, further we can spare the SlicePtr indirection
pub enum OpCode<#[AllocLifetime] 'b> {
    Lit(SlicePtr<'b,u8>, LitDesc),                                  //A opcode that produces a literal
    Let(Ptr<'b, Exp<'b>>),                                          //A Subsope that computes some values and returns them (intermidiary values are removed)
    Unpack(ValueRef),                                               //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Switch(ValueRef, SlicePtr<'b,Ptr<'b, Exp<'b>>>),                //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Pack(Tag, SlicePtr<'b,ValueRef>),                               //Generates a value
    Invoke(FunDesc, SlicePtr<'b,ValueRef>),                         //Invokes a Function
    Try(Ptr<'b, Exp<'b>>, SlicePtr<'b,(Error, Ptr<'b, Exp<'b>>)>),  //Executes a try block and on error reverts to execute the corresponding catch Block
    Get(ValueRef, u8),                                              //gets a field of a single ctr adt
}


//A placeholder for a generic value in a type builder
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Parsable, Serializable, VirtualSize)]
pub struct TypeInputRef(pub u8);

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

    NativeType {
        caps: CapSet,
        typ: NativeType,
        applies: SlicePtr<'a, Ptr<'a, RuntimeType<'a>>>
    },

    NewType {
        txt: Hash,
        offset: u8,
    },

    AccountType {
        address: Hash,
    },
}

//A type identifier in a type builder
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeKind {
    Custom {
        module: Hash,
        offset: u8,
    },

    Native {
        typ:NativeType,
    }
}

#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeBuilder<#[AllocLifetime] 'b> {
    Dynamic(CapSet, TypeKind, SlicePtr<'b,(bool, Ptr<'b, TypeBuilder<'b>>)>),
    Ref(TypeInputRef),
}

//an element in the backing store
#[derive(Clone, Copy, Eq, PartialEq, Debug, Parsable, Serializable)]
pub struct StoreElem<#[AllocLifetime] 'a> {
    pub val: Ptr<'a, Object<'a>>,
    pub typ: Ptr<'a, RuntimeType<'a>>
}


//A Object/Adt at runtime
#[derive(Eq, PartialEq, Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum Object<#[AllocLifetime] 'a> {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    I128(i128),
    U128(u128),
    Data(SlicePtr<'a,u8>),
    Adt(u8, SlicePtr<'a,Ptr<'a,Object<'a>>>),
    Context(u64),                               //u64 == counter for uniques   //rest environment
    //Note: Unique, Singleton, PrimaryKey, SecondaryKey are represented as Data(20)
}

