use alloc::prelude::*;
use encoding::*;
use errors::*;

//A Simple 160bit hash
pub type Hash = [u8;20];

//Represents a reference to the nTh element on the stack from the top (Bernouli Index)
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct ValueRef(pub u16);

//Links that point to components in the storage
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub enum ModuleLink{
    Remote(Hash),
    This(Hash),
}

impl ModuleLink {
    pub fn to_hash(&self) -> Hash {
        match *self {
            ModuleLink::Remote(h) => h,
            ModuleLink::This(h) => h,
        }
    }
}

//All the Available Native types
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
pub enum NativeType {
    Data(u16),          //An arbitrary (up to u16:max_value()) sized data type
    SInt(u8),           // A signed integer avaiable in different sizes (powers o3 bytes up to 16Bytes)
    UInt(u8),           // A unsigned integer avaiable in different sizes (powers o3 bytes up to 16Bytes)
    Bool,               // A Boolean with two ctrs True & False
    Tuple(u8),          // A And Type with up to u8::max_value() fields
    Alternative(u8),    // A Or Type with up to u8::max_value() ctrs
    Context,            // A Context gives access to blockchain constants and allows to generate unique numbers
    Unique,             // A Unique value (exist only once as it is linear so Eq always returns false)
    Singleton,          // A Unique value that is assosiated wit a new type (meaning only one Singleton[T] exists per T)
    Index,              // A Index that represents a storage slot on the blockchain
    Ref,                // A references that points ot  a storage slot on the blockchain
}


//All the Available Native Funcitions
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable)]
#[repr(u8)]
pub enum NativeFunc {
    And,        //Deploys a logical and on bools or bitwise on ints
    Or,         //Deploys a logical or on bools or bitwise on ints
    Xor,        //Deploys a logical xor on bools or bitwise on ints
    Not,        //Deploys a logical not on bools or bitwise on ints
    Extend,     //Increases the size of a int without changing its value
    Cut,        //Decreases the size of an int without changing its value (throws if not possible)
    SignCast,   //transforms a signed int to an unsigned and vice versa without changing its value (throws if not possible)
    Add,        //Does an arithmetic addition of two ints (throws on under or overflow)
    Sub,        //Does an arithmetic subtraction of two ints (throws on under or overflow)
    Mul,        //Does an arithmetic multiplication of two ints (throws on under or overflow)
    Div,        //Does an arithmetic dividation of two ints (throws on a division by zero)
    Eq,         //Compares two types for equality
    Hash,       //Calculates the hash of a val is structurally encoded
    PlainHash,  //Calculates the hash of a plain data primitive without special encoding
    Lt,         //Compares two values to decide if one is less than the other
    Gt,         //Compares two values to decide if one is greater than the other
    Lte,        //Compares two values to decide if one is less than or equal the other
    Gte,        //Compares two values to decide if one is greater or equal than the other
    ToData,     //Converts numbers, uniques, singleton, refs, indexes to data
    Concat,     //Concatenates two data values
    SetBit,     //Sets a bit in a data value to 1/0
    GetBit,     //Checks if a bit in a data value is 1/0
    ToUnique,   //Generates a Unique from a singleton (consuming it in the process)
    GenUnique,  //Generates a new Unique from the context (generating a new context as well)
    FullHash,   //Gets the Hash of the full transaction (including the Signatures) - from Context
    TxTHash,    //Gets the Hash of the transaction (not including te signatures) - from Context
    CodeHash,   //Gets the Hash of the code only - from Context
    BlockNo,    //The BlockNo of the block containing the transaction  - from Context
    GenIndex,   //Generates a index from either a unique or plain data (consumes the value allowing to generate unique indexes)
    ToRef,      //Generates a ref from either an index or plain data
    Derive,     //Combines 2 indexes or 2 refs to a new one allowing derive indexes & refs deterministically
}

//All the Available Native Errors
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
#[repr(u8)]
pub enum NativeError {
    NumericError,   //Thrown when a numeric operation fails or is undefined
    IndexError,
}

//Most vectors used in sanskrit have a max len of 255, in some places more is needed
#[derive(Eq, PartialEq, Hash, Clone, Debug, Parsable, Serializable)]
pub struct LargeVec<T>(pub Vec<T>);

//Represents a Identifier for a constructor of a specific Adt
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct Tag(pub u8);

//All the Available Native Capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Parsable, Serializable)]
#[repr(u8)]
pub enum NativeCap {
    Drop,       //Indicates if the value can be dropped (requires that nested values can be droped as well)
    Copy,       //Indicates if the value can be copied (requires that nested values can be copied as well)
    Persist,    //Indicates if the value can be persisted (requires that nested values can be persisted as well)
    Consume,    //Indicates if the value can be unpacked (implies inspect)
    Inspect,    //Indicates if the nested values can be inspected
    Embed,      //Indicates if the value can be embeded into another value
    Create,     //Indicates if the value can be created
    Indexed,    //Indicates if the value contains a storage index
}

//the Parsable Derive is special for the next two see encoding
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Ptr<'a, T>(pub &'a T);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct SlicePtr<'a, T>(pub &'a [T]);

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct MutSlicePtr<'a, T>(pub &'a mut [T]);