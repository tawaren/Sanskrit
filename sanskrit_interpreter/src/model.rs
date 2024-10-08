use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;

// AllocParsable is a workaround to Parsable as AllocLifetime stopped working (it takes the first generic param as AllocLifetime)
//A Block
#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub struct Exp</*#[AllocLifetime]*/ 'b>(pub SlicePtr<'b, OpCode<'b>>);

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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Parsable, Serializable, VirtualSize)]
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

#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub enum OpCode</*#[AllocLifetime]*/ 'b> {
    Void,                                                          //A opcode that produces something that is never used
    Data(SlicePtr<'b,u8>),                                         //A opcode that produces a literal
    SpecialLit(SlicePtr<'b,u8>, LitDesc),                          //An opcode that produces an external literal
    Let(Ptr<'b, Exp<'b>>),                                         //A Subsope that computes some values and returns them (intermidiary values are removed)
    Unpack(ValueRef),                                              //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Switch(ValueRef, SlicePtr<'b,Ptr<'b, Exp<'b>>>),               //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Pack(Tag, SlicePtr<'b,ValueRef>),                              //Generates a value
    CreateSig(u16, SlicePtr<'b,ValueRef>),                         //Captures a Function Pointer
    InvokeSig(ValueRef, SlicePtr<'b,ValueRef>),                    //Invokes a Function Pointer
    Invoke(u16, SlicePtr<'b,ValueRef>),                            //Invokes a Function
    RepeatedInvoke(u16, SlicePtr<'b,ValueRef>, ValueRef, Tag, u8), //Invokes a Function repeatedly
    Try(Ptr<'b, OpCode<'b>>, Ptr<'b, Exp<'b>>, Ptr<'b, Exp<'b>>),  //Executes the Opcode, and then runs either exp depending on sucess or failure
    Rollback,
    Return(SlicePtr<'b,ValueRef>),
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
    SysInvoke(u8, SlicePtr<'b,ValueRef>),
    TypedSysInvoke(u8, Kind, SlicePtr<'b,ValueRef>),
    #[cfg(feature = "dynamic_gas")]
    ConsumeGas(u32)                                                 //Consumes some additional gas
}

#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub struct TransactionDescriptor</*#[AllocLifetime]*/ 'c> {
    #[ByteSize]
    pub byte_size:Option<usize>,
    #[VirtualSize]
    pub virt_size:Option<usize>,
    pub gas_cost:u32,
    pub max_stack:u16,
    pub max_frames:u16,
    pub max_mem:u16,
    pub params:SlicePtr<'c,TxTParam<'c>>,
    pub returns:SlicePtr<'c,TxTReturn<'c>>,
    #[cfg(feature = "dynamic_gas")]
    pub functions:SlicePtr<'c,TxTFunction<'c>>,        //multiple functions (calls are embeded)
    #[cfg(not(feature = "dynamic_gas"))]
    pub functions:SlicePtr<'c,Ptr<'c, Exp<'c>>>,        //multiple functions (calls are embeded)
}

#[cfg(feature = "dynamic_gas")]
#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub struct TxTFunction</*#[AllocLifetime]*/ 'c> {
    pub gas:u32,
    pub body:Ptr<'c, Exp<'c>>
}

#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub struct TxTParam</*#[AllocLifetime]*/ 'c> {
    pub primitive:bool,
    pub copy:bool,
    pub drop:bool,
    pub consumes:bool,
    pub typ:Ptr<'c, RuntimeType<'c>>,
    pub desc:Ptr<'c, ValueSchema<'c>>
}

#[derive(Copy, Clone, Debug, AllocParsable, Serializable, VirtualSize)]
pub struct TxTReturn</*#[AllocLifetime]*/ 'c> {
    pub primitive:bool,
    pub copy:bool,
    pub drop:bool,
    pub typ:Ptr<'c, RuntimeType<'c>>,
    pub desc:Ptr<'c, ValueSchema<'c>>
}

//The Option<Hash> are type indexes and field indexes respectively
#[derive(Clone, Copy, Eq, PartialEq, Debug, AllocParsable, Serializable, VirtualSize)]
pub enum ValueSchema</*#[AllocLifetime]*/ 'a> {
    Adt(Option<Ptr<'a,(Hash,u8)>>, SlicePtr<'a, SlicePtr<'a, (SlicePtr<'a,u8>, Ptr<'a, ValueSchema<'a>>)>>),
    Data(u16),
    Unsigned(u8),
    Signed(u8)
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, AllocParsable, Serializable, VirtualSize)]
pub enum RuntimeType</*#[AllocLifetime]*/ 'a> {
    Custom {
        module: Hash,
        offset: u8,
        applies: SlicePtr<'a, Ptr<'a, RuntimeType<'a>>>
    },

    Projection {
        depth:u8,
        typ:  Ptr<'a, RuntimeType<'a>>
    },

    Virtual {
        id: Hash
    }
}

#[derive(Copy, Clone, VirtualSize)]
pub union Entry<'a> {
    pub data: SlicePtr<'a, u8>,
    pub adt: Adt<'a>,
    pub func: Func<'a>,
    pub u8: u8,
    pub i8: i8,
    pub u16: u16,
    pub i16: i16,
    pub u32: u32,
    pub i32: i32,
    pub u64: u64,
    pub i64: i64,
    pub u128: u128,
    pub i128: i128,
}

#[derive(Copy, Clone, VirtualSize)]
pub struct Adt<'a>(pub u8, pub SlicePtr<'a, Entry<'a>>);

#[derive(Copy, Clone, VirtualSize)]
pub struct Func<'a>(pub u16, pub SlicePtr<'a, Entry<'a>>);
