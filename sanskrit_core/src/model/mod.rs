pub mod resolved;
pub mod linking;

use sanskrit_common::capabilities::*;
use alloc::prelude::*;
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
    pub adts: Vec<AdtComponent>,
    //A Module has Functions
    pub functions: Vec<FunctionComponent>,
    //A Module has Errors (they are interchangeable so just the number matters)
    pub errors: u8,
}

//Represents an Adt
#[derive(Debug, Parsable, Serializable)]
pub struct AdtComponent {
    pub provided_caps:CapSet,                   //All caps that this is allowed to have (not considering generics)
    pub generics:Vec<Generic>,                  //An Adt has Generic Type parameters that can be constraint
    pub import:PublicImport,                    //An Adt has imports usable in its constructors
    pub constructors:Vec<Case>                  //An Adt has multiple Constructors
}

//Represents a Function (Like a function but has the ability to consume its arguments and return borrows)
#[derive(Debug, Parsable, Serializable)]
pub struct FunctionComponent {
    pub generics:Vec<Generic>,                  //A Function has Generic Type parameters that can be constraint
    pub visibility:Visibility,                  //A Function has a visibility defining who can call it
    pub import:BodyImport,                      //A Function has imports usable in its body/code
    pub risk: BTreeSet<ErrorRef>,               //A Function can produce errors which are declared as risks
    pub params:Vec<Param>,                      //A Function has input params
    pub returns:Vec<Ret>,                       //A Function has returns
    pub code: Exp                               //A Function has a body Expression
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

//As Base import but allows to import functions
#[derive(Debug, Parsable, Serializable)]
pub struct BodyImport {
    pub base:PublicImport,
    pub functions:Vec<FunctionImport>,         //Function Imports
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
pub enum FunctionImport {
    Module(FuncLink, Vec<TypeRef>),            //A Function Imported From A module
    Native(NativeFunc, Vec<TypeRef>),          //A Native Function
}

//Represents an imported Error
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash, Debug, Parsable, Serializable)]
pub enum ErrorImport {
    Module(ErrorLink),                          //An Error imported From A Module
    Native(NativeError)                         //A Native Error
}

//Represents an imported Adt/Native including the application of generic type parameters
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable)]
pub enum BaseType {
    Module(AdtLink),               //An Adt imported From A Module
    Native(NativeType)             //An Native base type
}

//Represents a type construction
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Parsable, Serializable)]
pub enum Type {
    Real(BaseType, Vec<TypeRef>),                  //A type consisting of a base and some capabilities
    Generic(GenRef)                                        //A type consisting of a generic base and some capabilities
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
pub struct AdtLink{pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Parsable, Serializable)]
pub struct CapLink{pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct FuncLink {pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ErrorLink{pub module:ModRef, pub offset:u8}


//An Expression that either returns a value or aborts with an error
#[derive(Debug, Parsable, Serializable)]
pub enum Exp {
    Ret(LargeVec<OpCode>, Vec<ValueRef>, Vec<ValueRef>),
    Throw(ErrorRef),
}

//All the Opcodes
#[derive(Debug, Parsable, Serializable)]
pub enum OpCode {
    Lit(LargeVec<u8>, TypeRef),                         //A opcode that produces a literal
    Let(Exp),                                           //A Subsope that computes some values and returns them (intermidiary values are removed)
    CopyFetch(ValueRef),                                //Copies a value (Needs Copy Cap)
    Fetch(ValueRef),                                    //Moves a value onto the top of the stack (allows to move it into a branch)
    BorrowFetch(ValueRef),                              //Borrows a value onto the top of the stack (allows to borrow it into a branch)
    Free(ValueRef),                                     //Releases a Borrow and unlocks the borrowed elements
    Drop(ValueRef),                                     //Drops a value (Needs Drop Capability or a Field Less Leave type with Consume cap)
    BorrowUnpack(ValueRef, TypeRef),                    //Inspects a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Unpack(ValueRef, TypeRef),                          //Consumes a value to produce its fields (single Ctr only) (Needs Consume or Inspect Cap)
    Field(ValueRef, TypeRef, u8),                       //<-- requires Consume & others to have Drop
    CopyField(ValueRef, TypeRef, u8),                   //<-- requires Inspect & Field to have Copy
    BorrowField(ValueRef, TypeRef, u8),                 //<-- requires Inspect
    BorrowSwitch(ValueRef, TypeRef, Vec<Exp>),          //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Switch(ValueRef, TypeRef, Vec<Exp>),                //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    BorrowPack(TypeRef, Tag, Vec<ValueRef>),            //Generates a value for a multi ctr mutli field type (which is immideately borrowed)
    Pack(TypeRef, Tag, Vec<ValueRef>),                  //Generates a value for a multi ctr mutli field type
    CopyPack(TypeRef, Tag, Vec<ValueRef>),              //Generates a value for a multi ctr mutli field type by coping the inputs (requieres copy cap for them)
    Invoke(FuncRef, Vec<ValueRef>),                     //Invokes a Function
    Try(Exp, Vec<(ErrorRef, Exp)>),                     //Executes a try block and on error reverts to execute the corresponding catch Block
}



