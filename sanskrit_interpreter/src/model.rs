use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;
use sanskrit_common::capabilities::CapSet;


//A type to represent an error in the interpreter
#[derive(Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct Error(pub u16);

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
    Id,
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

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum OpCode<#[AllocLifetime] 'b> {
    Lit(SlicePtr<'b,u8>),                                           //A opcode that produces a literal
    SpecialLit(SlicePtr<'b,u8>, LitDesc),                           //An opcode that produces an external literal
    Let(Ptr<'b, Exp<'b>>),                                          //A Subsope that computes some values and returns them (intermidiary values are removed)
    Unpack(ValueRef),                                               //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Switch(ValueRef, SlicePtr<'b,Ptr<'b, Exp<'b>>>),                //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Pack(Tag, SlicePtr<'b,ValueRef>),                               //Generates a value
    Invoke(u16, SlicePtr<'b,ValueRef>),                             //Invokes a Function
    Try(Ptr<'b, Exp<'b>>, SlicePtr<'b,(Error, Ptr<'b, Exp<'b>>)>),  //Executes a try block and on error reverts to execute the corresponding catch Block
    Get(ValueRef, u8),                                              //gets a field of a single ctr adt
    And(ValueRef,ValueRef),                                         //Deploys a logical and on bools or bitwise on ints
    Or(ValueRef,ValueRef),                                          //Deploys a logical or on bools or bitwise on ints
    Xor(ValueRef,ValueRef),                                         //Deploys a logical xor on bools or bitwise on ints
    Not(ValueRef),                                                  //Deploys a logical not on bools or bitwise on ints
    ToU(u8,ValueRef),                                               //cast to an Unsigned Integer with u8 Bytes
    ToI(u8,ValueRef),                                               //cast to an Signed Integer with u8 Bytes
    Add(ValueRef,ValueRef),                                         //Does an arithmetic addition of two ints (throws on under or overflow)
    Sub(ValueRef,ValueRef),                                         //Does an arithmetic subtraction of two ints (throws on under or overflow)
    Mul(ValueRef,ValueRef),                                         //Does an arithmetic multiplication of two ints (throws on under or overflow)
    Div(ValueRef,ValueRef),                                         //Does an arithmetic dividation of two ints (throws on a division by zero)
    Eq(ValueRef,ValueRef),                                          //Compares two values for equality
    Hash(ValueRef),                                                 //Calculates a plain hash for a data input (not structurally encoded)
    Lt(ValueRef,ValueRef),                                          //Compares two values to decide if one is less than the other
    Gt(ValueRef,ValueRef),                                          //Compares two values to decide if one is greater than the other
    Lte(ValueRef,ValueRef),                                         //Compares two values to decide if one is less than or equal the other
    Gte(ValueRef,ValueRef),                                         //Compares two values to decide if one is greater or equal than the other
    ToData(ValueRef),                                               //Transforms Integers & Uniques to data
    Derive(ValueRef,ValueRef),                                      //derives a new index or referenz from two others
    //Gas Testing Operands
    Id(ValueRef),                                                   //Makes a Copy of the input (this is for testing) -- Establishes a Baseline
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
    //Note: Unique, Singleton, PrimaryKey, SecondaryKey are represented as Data(20)
}

#[derive(Copy, Clone, Debug, Parsable, Serializable)]
pub struct AdtDescriptor<#[AllocLifetime] 'b> {
    pub generics:SlicePtr<'b, TypeTypeParam>,
    pub constructors:SlicePtr<'b,SlicePtr<'b,TypeBuilder<'b>>>,
    pub base_caps:CapSet,                   //the allowed capabilities
    pub id: AdtId,
}

#[derive(Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct FunctionDescriptor<#[AllocLifetime] 'b> {
    pub gas_cost:u32,
    pub max_stack:u16,
    pub max_frames:u16,
    pub generics:SlicePtr<'b,FunTypeParam>,
    pub params:SlicePtr<'b,Param<'b>>,
    pub returns:SlicePtr<'b,Return<'b>>,
    pub functions:SlicePtr<'b,Ptr<'b, Exp<'b>>>,        //multiple functions (calls are embeded)
}

//an identifier of an adt
#[derive(Copy, Clone, Debug, Parsable, Serializable)]
pub struct AdtId {
    pub module:Hash,        //Hash is module Hash
    pub offset:u8
}


//todo: Make structural
//bool == is is_phantom
#[derive(Copy, Clone, Eq, PartialEq, Debug, Parsable, Serializable, VirtualSize)]
pub struct TypeTypeParam(pub bool, pub CapSet);

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

#[derive(Clone, Copy, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeBuilder<#[AllocLifetime] 'b> {
    Dynamic(CapSet, TypeKind, SlicePtr<'b,(bool, Ptr<'b, TypeBuilder<'b>>)>),
    Ref(TypeInputRef),
    Image(Ptr<'b, TypeBuilder<'b>>),
}

//A placeholder for a generic value in a type builder
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Parsable, Serializable, VirtualSize)]
pub struct TypeInputRef(pub u8);

//A type identifier in a type builder
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub enum TypeKind {
    //todo: split in Adt & Sig
    Custom {
        module: Hash,
        offset: u8,
    }
}