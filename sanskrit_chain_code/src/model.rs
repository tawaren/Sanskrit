use alloc::boxed::Box;
use alloc::vec::Vec;
use sanskrit_common::model::*;
use sanskrit_common::encoding::*;

//A Block
#[derive(Clone, Debug, Parsable, Serializable)]
pub struct Exp(pub Vec<OpCode>);

//Description for a literal type
#[derive(Copy, Clone, Eq, PartialEq, Debug, Parsable, Serializable)]
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Parsable, Serializable)]
#[repr(u8)]
pub enum Kind {
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
    Data
}

#[derive(Clone, Debug, Parsable, Serializable)]
pub enum OpCode {
    Void,                                                          //A opcode that produces something that is never used
    Data(LargeVec<u8>),                                            //A opcode that produces a literal
    SpecialLit(LargeVec<u8>, LitDesc),                             //An opcode that produces an external literal
    Let(Box<Exp>),                                                      //A Subsope that computes some values and returns them (intermidiary values are removed)
    Unpack(ValueRef),                                              //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Switch(ValueRef, Vec<Exp>),                                    //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Pack(Tag, Vec<ValueRef>),                                      //Generates a value
    CreateSig(u16, Vec<ValueRef>),                                 //Captures a Function Pointer
    InvokeSig(ValueRef, Vec<ValueRef>),                            //Invokes a Function Pointer
    Invoke(u16, Vec<ValueRef>),                                    //Invokes a Function
    RepeatedInvoke(u16, Vec<ValueRef>, ValueRef, Tag, u8),         //Invokes a Function repeatedly
    Try(Box<OpCode>, Box<Exp>, Box<Exp>),                          //Executes the Opcode, and then runs either exp depending on sucess or failure
    Rollback,
    Return(Vec<ValueRef>),
    Get(ValueRef, u8),                                              //gets a field of a single ctr adt
    And(Kind, ValueRef,ValueRef),                                   //Deploys a logical and on bools or bitwise on ints | data
    Or(Kind, ValueRef, ValueRef),                                   //Deploys a logical or on bools or bitwise on ints | data
    Xor(Kind, ValueRef, ValueRef),                                  //Deploys a logical xor on bools or bitwise on ints | data
    Not(Kind, ValueRef),                                            //Deploys a logical not on bools or bitwise on ints | data
    //ToU(u8,ValueRef),                                             //cast to an Unsigned Integer with u8 Bytes
    //ToI(u8,ValueRef),                                             //cast to an Signed Integer with u8 Bytes
    Add(Kind, ValueRef,ValueRef),                                   //Does an arithmetic addition of two ints (throws on under or overflow)
    Sub(Kind, ValueRef,ValueRef),                                   //Does an arithmetic subtraction of two ints (throws on under or overflow)
    Mul(Kind, ValueRef,ValueRef),                                   //Does an arithmetic multiplication of two ints (throws on under or overflow)
    Div(Kind, ValueRef,ValueRef),                                   //Does an arithmetic dividation of two ints (throws on a division by zero)
    Eq(Kind, ValueRef,ValueRef),                                    //Compares two values for equality
    Lt(Kind, ValueRef,ValueRef),                                    //Compares two values to decide if one is less than the other
    Gt(Kind, ValueRef,ValueRef),                                    //Compares two values to decide if one is greater than the other
    Lte(Kind, ValueRef,ValueRef),                                   //Compares two values to decide if one is less than or equal the other
    Gte(Kind, ValueRef,ValueRef),                                   //Compares two values to decide if one is greater or equal than the other
    ToData(Kind, ValueRef),                                         //Transforms Integers & Uniques to data
    FromData(Kind, ValueRef),                                       //Transforms Data to Integers & Uniques
    //Gas Testing Operands
    Id(ValueRef),                                                   //Makes a Copy of the input (this is for testing) -- Establishes a Baseline
    SysInvoke(u8, Vec<ValueRef>),
    TypedSysInvoke(u8, Kind, Vec<ValueRef>),
}

#[derive(Clone, Debug, Parsable, Serializable)]
pub struct TransactionDescriptor {
    #[ByteSize]
    pub byte_size:Option<usize>,
    pub params:Vec<TxTParam>,
    pub returns:Vec<TxTReturn>,
    pub functions:Vec<Exp>,             //multiple functions (calls are embeded)
}

#[derive(Clone, Debug, Parsable, Serializable)]
pub struct TxTParam {
    pub primitive:bool,
    pub copy:bool,
    pub drop:bool,
    pub consumes:bool,
    pub typ:Box<RuntimeType>,
    pub desc:Box<ValueSchema>
}

#[derive(Clone, Debug, Parsable, Serializable)]
pub struct TxTReturn {
    pub primitive:bool,
    pub copy:bool,
    pub drop:bool,
    pub typ:Box<RuntimeType>,
    pub desc:Box<ValueSchema>
}

//The Option<Hash> are type indexes and field indexes respectively
#[derive(Clone, Eq, PartialEq, Debug, Parsable, Serializable)]
pub enum ValueSchema {
    Adt(Option<Box<(Hash,u8)>>, Vec<Vec<(Vec<u8>, Box<ValueSchema>)>>),
    Data(u16),
    Unsigned(u8),
    Signed(u8)
}

#[derive(Clone, Eq, PartialEq, Debug, Parsable, Serializable)]
pub enum RuntimeType {
    Custom {
        module: Hash,
        offset: u8,
        applies: Vec<RuntimeType>
    },

    Projection {
        depth:u8,
        typ: Box<RuntimeType>
    },

    Virtual {
        id: Hash
    }
}

