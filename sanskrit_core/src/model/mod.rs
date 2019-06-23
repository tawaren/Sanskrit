pub mod resolved;
pub mod linking;

use sanskrit_common::capabilities::*;
use alloc::vec::Vec;
use alloc::collections::BTreeSet;
use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::errors::*;

//Represents a Module
#[derive(Debug, Parsable, Serializable)]
pub struct Module {
    //A Module has compiler specific meta information (Not of concern to Sanskrit)
    pub meta: LargeVec<u8>,
    //A Module has Adts
    pub data: Vec<DataComponent>,
    //A Module has Sigs
    pub sigs: Vec<SigComponent>,
    //A Module has Functions
    pub functions: Vec<FunctionComponent>,
    //A Module has Errors (they are interchangeable so just the number matters)
    pub errors: u8,
}

#[derive(Debug, Parsable, Serializable)]
pub enum DataImpl {
    Adt{constructors:Vec<Case>},               //An Adt has multiple Constructors
    Lit(u16),
    ExternalAdt(u16),
    ExternalLit(u16,u16),                     // second u16 is size
}

//Represents an Adt
#[derive(Debug, Parsable, Serializable)]
pub struct DataComponent {
    pub provided_caps:CapSet,                   //All caps that this is allowed to have (not considering generics)
    pub generics:Vec<Generic>,                  //An Adt has Generic Type parameters that can be constraint
    pub import:PublicImport,                    //An Adt has imports usable in its constructors
    pub body:DataImpl
}

//Represents a fun signature
#[derive(Debug, Parsable, Serializable)]
pub struct FunSigShared {
    pub generics:Vec<Generic>,                  //A Fun/Sig has Generic Type parameters that can be constraint
    pub import:PublicImport,                    //A Fun/Sig has imports usable in its dig & body
    pub risk: BTreeSet<ErrorRef>,               //A Fun/Sig can produce errors which are declared as risks
    pub params:Vec<Param>,                      //A Fun/Sig has input params
    pub returns:Vec<Ret>,                       //A Fun/Sig has returns
}

//Represents a fun signature
#[derive(Debug, Parsable, Serializable)]
pub struct SigComponent {
    pub shared:FunSigShared,
    pub provided_caps:CapSet,             //All caps that this is allowed to have (not considering captured generics)
    pub local_generics:Vec<u8>            //A Sig have local generics

}

#[derive(Debug, Parsable, Serializable)]
pub enum FunctionImpl{
    External(u16),
    Internal{
        functions:Vec<FunctionImport>,         //Function Imports
        code: Exp                              //A Function has a body Expression
    }
}

//Represents a Function (Like a function but has the ability to consume its arguments and return borrows)
#[derive(Debug, Parsable, Serializable)]
pub struct FunctionComponent {
    pub shared:FunSigShared,
    pub visibility:Visibility,                 //A Fun has a visibility defining who can call it
    pub implements:Vec<SigImpl>,
    pub body:FunctionImpl
}

//Represents a Generic
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable)]
pub enum Generic {
    Phantom,
    Physical(CapSet),
}

//Represents a set of imports and type constructions usable in signatures
#[derive(Debug, Parsable, Serializable)]
pub struct PublicImport {
    pub modules:Vec<ModuleLink>,               //Module Imports
    pub errors:Vec<ErrorImport>,               //Error Imports
    pub types:Vec<Type>,                       //Type Imports
}

//Represents a Sig implementation
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub struct SigImpl{
    pub typ:TypeRef,         //Defines the Type of this Impl (Must point to a SigImport)
    pub captures:Vec<u8>     //Defines which of the signatures params are captured
}


//Represents a Function Param
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash, Debug, Parsable, Serializable)]
pub struct Param{
    pub consumes:bool,          //Defines if the Param is consumed by the Function
    pub typ:TypeRef             //Defines the Type of the Param
}

//Represents a Function Return
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub struct Ret{
    pub borrows:Vec<ValueRef>,  //Defines if the Return has borrowed other values and which
    pub typ:TypeRef             //Defines the Type of the Return
}

//Represents a constructor of an adt
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub struct Case {
    pub fields:Vec<TypeRef>     //The Type of the constructors fields
}

//Represents the visibility of a Function
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub enum Visibility{
    Private,                    //A Visibility restricting invokes to the same Module
    Protected(Vec<GenRef>),     //A Visibility restricting invokes to Modules that includes one of the generic types
    Public,                     //A Visibility allowing everybody to invoke
}

//Represents an imported Funcfomer
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub struct FunctionImport {
    pub link:FuncLink,
    pub applies:Vec<TypeRef>,
}

//Represents an imported Error
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash, Debug, Parsable, Serializable)]
pub struct ErrorImport{
    pub link:ErrorLink
}



//Represents an imported Adt/Native including the application of generic type parameters
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable)]
pub enum BaseType {
    Sig(SigLink),               //A Sig imported From A Module
    Data(DataLink),             //An Adt or Lit imported From A Module
}


//Represents a type construction
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Parsable, Serializable)]
pub enum Type {
    Real(BaseType, Vec<TypeRef>),                  //A type consisting of a base and some capabilities
    Image(TypeRef),                                //A Wrapper Type that captures the information of another type but not the value
    Generic(GenRef)                                //A type consisting of a generic base and some capabilities
}

//References that point to components in the input
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ModRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct TypeRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct GenRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct FuncRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ErrorRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct SigLink{pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct DataLink {pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Parsable, Serializable)]
pub struct CapLink{pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct FuncLink {pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ErrorLink{pub module:ModRef, pub offset:u8}


//An Expression that either returns a value or aborts with an error
#[derive(Debug, Parsable, Serializable)]
pub enum Exp {
    Ret(LargeVec<OpCode>, Vec<ValueRef>),
    Throw(ErrorRef),
}

//All the Opcodes
#[derive(Debug, Parsable, Serializable)]
pub enum OpCode {
    Lit(LargeVec<u8>, TypeRef),                         //A opcode that produces a literal
    Let(Exp),                                           //A Subsope that computes some values and returns them (intermidiary values are removed)
    CopyFetch(ValueRef),                                //Copies a value (Needs Copy Cap)
    BorrowFetch(ValueRef),                              //Borrows a value onto the top of the stack (allows to borrow it into a branch)
    Fetch(ValueRef),                                    //Moves a value onto the top of the stack (allows to move it into a branch)
    Discard(ValueRef),                                  //Releases a Borrow and unlocks the borrowed elements or drops a value with Drop capability
    DiscardMany(Vec<ValueRef>),                         //As discard but many elements
    DiscardBorrowed(ValueRef, Vec<ValueRef>),           //Steals the borrows of a value and Frees it (only works if the stealing value borrows)
    CopyUnpack(ValueRef, TypeRef),                      //Copies a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    BorrowUnpack(ValueRef, TypeRef),                    //Inspects a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Unpack(ValueRef, TypeRef),                          //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    CopyField(ValueRef, TypeRef, u8),                   //<-- requires Inspect & Field to have Copy
    BorrowField(ValueRef, TypeRef, u8),                 //<-- requires Inspect
    Field(ValueRef, TypeRef, u8),                       //<-- requires Consume & others to have Drop
    CopySwitch(ValueRef, TypeRef, Vec<Exp>),            //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit CopyUnpack)
    BorrowSwitch(ValueRef, TypeRef, Vec<Exp>),          //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit BorrowUnpack)
    Switch(ValueRef, TypeRef, Vec<Exp>),                //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    CopyPack(TypeRef, Tag, Vec<ValueRef>),              //Generates a value for a multi ctr mutli field type by coping the inputs (requieres copy cap for them)
    BorrowPack(TypeRef, Tag, Vec<ValueRef>),            //Generates a value for a multi ctr mutli field type (which is immideately borrowed)
    Pack(TypeRef, Tag, Vec<ValueRef>),                  //Generates a value for a multi ctr mutli field type
    Invoke(FuncRef, Vec<ValueRef>),                     //Invokes a Function
    Try(Exp, Vec<(ErrorRef, Exp)>),                     //Executes a try block and on error reverts to execute the corresponding catch Block
    //ModuleIndex,                                        //A private constant index associated with each module (it returns the one of the current Module)
    Image(ValueRef),                                    //Produces the Image of the input withoutconsuming it
    ExtractImage(ValueRef),                             //Removes a layer from a nested Image: Image[Image[T]] => Image[T]
    CreateSig(FuncRef, u8, Vec<ValueRef>),
    InvokeSig(ValueRef, TypeRef, Vec<ValueRef>)
}



