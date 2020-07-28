pub mod resolved;
pub mod linking;
pub mod bitsets;
pub mod efficency;

use alloc::vec::Vec;
use sanskrit_common::model::*;
use sanskrit_common::encoding::*;
use sanskrit_common::encoding::Serializable;
use sanskrit_common::errors::*;
use model::resolved::ResolvedType;
use utils::Crc;
use model::bitsets::{CapSet, PermSet};

//Represents a Module
#[derive(Debug, Parsable, Serializable)]
pub struct Module {
    #[ByteSize]
    pub byte_size:Option<usize>,
    //A Module has compiler specific meta information (Not of concern to Sanskrit)
    pub meta: LargeVec<u8>,
    //A Module has Adts
    pub data: Vec<DataComponent>,
    //A Module has Sigs
    pub sigs: Vec<SigComponent>,
    //defines the order of the data components -- true means data, false means sig
    pub data_sig_order: BitSerializedVec,
    //A Module has Functions
    pub functions: Vec<FunctionComponent>,
    //A Module has Implementations
    pub implements: Vec<ImplementComponent>,
    //defines the order of the callable components -- true means function, false means implement
    pub fun_impl_order: BitSerializedVec,
}

#[derive(Debug, Parsable, Serializable)]
pub enum DataImpl {
    Internal {constructors:Vec<Case>},      //An Adt has multiple Constructors
    External(u16),                          // u16 is size
}

//Represents an Adt
#[derive(Debug, Parsable, Serializable)]
pub struct DataComponent {
    #[ByteSize]
    pub byte_size:Option<usize>,
    pub create_scope: Accessibility,
    pub consume_scope: Accessibility,
    pub inspect_scope: Accessibility,
    pub top:bool,
    pub provided_caps:CapSet,                   //All caps that this is allowed to have (not considering generics)
    pub generics:Vec<Generic>,                  //An Adt has Generic Type parameters that can be constraint
    pub import: PublicImport,                   //An Adt has imports usable in its constructors
    pub body:DataImpl
}

//Represents a fun signature
#[derive(Debug, Parsable, Serializable)]
pub struct FunSigShared {
    pub transactional:bool,                     //Marks this function as transactional
    pub generics:Vec<Generic>,                  //A Fun/Sig has Generic Type parameters that can be constraint
    pub import: PublicImport,                   //A Fun/Sig has imports usable in its dig & body
    pub params:Vec<Param>,                      //A Fun/Sig has input params
    pub returns:Vec<TypeRef>,                   //A Fun/Sig has returns
}

//Represents a fun signature
#[derive(Debug, Parsable, Serializable)]
pub struct SigComponent {
    #[ByteSize]
    pub byte_size:Option<usize>,
    pub call_scope: Accessibility,
    pub implement_scope: Accessibility,
    pub provided_caps:CapSet,             //All caps that this is allowed to have (not considering captured generics)
    pub shared:FunSigShared,
}

#[derive(Debug, Parsable, Serializable)]
pub enum CallableImpl {
    External,
    Internal{
        #[ByteSize]
        byte_size:Option<usize>,
        imports:BodyImport,       //Imports for use in code
        code: Exp                  //A Function has a body Expression
    }
}

//Represents a Function (Like a function but has the ability to consume its arguments and return borrows)
#[derive(Debug, Parsable, Serializable)]
pub struct FunctionComponent {
    #[ByteSize]
    pub byte_size:Option<usize>,
    pub scope: Accessibility,                 //A Fun has a visibility defining who can call it
    pub shared:FunSigShared,
    pub body: CallableImpl
}

#[derive(Debug, Parsable, Serializable)]
pub struct ImplementComponent {
    #[ByteSize]
    pub byte_size:Option<usize>,
    pub scope: Accessibility,                  //A Fun has a visibility defining who can call it
    pub sig: PermRef,                          //Defines the Type & Permission of this Implement (Must point to a Implement Perm)
    pub generics:Vec<Generic>,                 //A Fun/Sig has Generic Type parameters that can be constraint
    pub import: PublicImport,                  //A Fun/Sig has imports usable in its dig & body
    pub params:Vec<Param>,                     //A Fun/Sig has input params
    pub body: CallableImpl
}

//Note: the Serialize and Deserialize are in the efficency module
#[derive(Debug)]
pub struct BitSerializedVec(pub Vec<bool>);

//Represents a set of imports and type constructions usable in signatures
#[derive(Debug, Parsable, Serializable)]
pub struct PublicImport {
    pub modules:Vec<ModuleLink>,               //Module Imports
    pub types:Vec<TypeImport>,                 //Type Imports
}

//Represents a set of imports and type constructions usable in signatures
#[derive(Debug, Parsable, Serializable)]
pub struct BodyImport {
    pub public: PublicImport,
    pub callables:Vec<CallableImport>,       //Imports for use in code
    pub permissions:Vec<PermissionImport>,   //Permisions

}

pub enum Imports<'a> {
    Module(&'a Crc<ModuleLink>),
    Generics(&'a [Crc<ResolvedType>]),
    Public(&'a PublicImport),
    Body(&'a BodyImport)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub enum PermissionImport {
    Type(PermSet, TypeRef),
    Callable(PermSet, CallRef),
}

//Represesents a Callable (Function or Implement)
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub enum CallableImport {
    Function{link:FuncLink, applies:Vec<TypeRef>},
    Implement{link:ImplLink, applies:Vec<TypeRef>},
}

//Represents a Generic
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug, Parsable, Serializable)]
pub enum Generic {
    Phantom,
    Physical(CapSet),
}

//Represents a Function Param
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Hash, Debug, Parsable, Serializable)]
pub struct Param{
    pub consumes:bool,          //Defines if the Param is consumed by the Function
    pub typ:TypeRef             //Defines the Type of the Param
}

//Represents a constructor of an adt
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub struct Case {
    pub fields:Vec<TypeRef>     //The Type of the constructors fields
}

//Represents the visibility of a Function
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Hash, Debug, Parsable, Serializable)]
pub enum Accessibility {
    Local,                    //A Visibility restricting invokes to the same Module
    Guarded(Vec<GenRef>),     //A Visibility restricting invokes to Modules that includes one of the generic types
    Global,                     //A Visibility allowing everybody to invoke
}

//Represents a type construction
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Parsable, Serializable)]
pub enum TypeImport {
    Projection{ typ:TypeRef },
    Sig{link:SigLink, applies:Vec<TypeRef>},
    Data{link:DataLink, applies:Vec<TypeRef>},
    Virtual(Hash)                                  //Some type not under our control probably from the environment
}

//All the Available Capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Parsable, Serializable)]
#[repr(u8)]
pub enum Capability {
    Drop,       //Indicates if the value can be dropped (requires that nested values can be droped as well)
    Copy,       //Indicates if the value can be copied (requires that nested values can be copied as well)
    Persist,    //Indicates if the value can be persisted (requires that nested values can be persisted as well)
    Primitive,  //Indicates if the value can be passed from / to outside (unprotected)
    Value,      //Indicates that the value has no side effects when consumed (which would prevent its rollback)
    Unbound,    //Indicates that the value can be returned from a function / frame
}

//All the Available Permissions
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Parsable, Serializable)]
#[repr(u8)]
pub enum Permission {
    Create,
    Consume,
    Inspect,
    Call,
    Implement,
}


//References that point to components in the input
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ModRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct TypeRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct PermRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct GenRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct CallRef(pub u8);

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct SigLink{pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct DataLink {pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct FuncLink {pub module:ModRef, pub offset:u8}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ImplLink {pub module:ModRef, pub offset:u8}

//An Expression that either returns a value or aborts with an error
#[derive(Debug, Parsable, Serializable)]
pub struct Exp(pub LargeVec<OpCode>);

//All the Opcodes
//Note: The extra TypeRef & PermRef & CallRef -- are essential to limit memory consumption
//      Never remove them even if they can be constructed on the fly (like it would be the case for Project)
#[derive(Debug, Parsable, Serializable)]
pub enum OpCode {
    Lit(LargeVec<u8>, PermRef),                                             //A opcode that produces a literal
    Let(Exp),                                                               //A Subsope that computes some values and returns them (intermidiary values are removed)
    Copy(ValueRef),                                                         //Copies a value (Needs Copy Cap)
    Move(ValueRef),                                                         //Moves a value onto the top of the stack (allows to move it into a branch)
    Return(Vec<ValueRef>),
    Discard(ValueRef),                                                      //Releases a Borrow and unlocks the borrowed elements or drops a value with Drop capability
    DiscardMany(Vec<ValueRef>),                                             //As discard but many elements
    CopyUnpack(ValueRef, PermRef),                                          //Copies a value to produce its fields (single Ctr only) (Needs Inspect Cap)
    Unpack(ValueRef, PermRef),                                              //Consumes a value to produce its fields (single Ctr only) (Needs Consume Cap)
    CopyField(ValueRef, PermRef, u8),                                       //<-- requires Inspect & Field to have Copy
    Field(ValueRef, PermRef, u8),                                           //<-- requires Consume & others to have Drop
    CopySwitch(ValueRef, PermRef, Vec<Exp>),                                //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit CopyUnpack)
    Switch(ValueRef, PermRef, Vec<Exp>),                                    //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an implicit Unpack)
    Inspect(ValueRef, PermRef, Vec<Exp>),                                   //Branches on a type that has multiple ctrs where each branch corresponds to 1 Ctr (Does an a special read only unpack)
    CopyPack(PermRef, Tag, Vec<ValueRef>),                                  //Generates a value for a multi ctr mutli field type by coping the inputs (requieres copy cap for them)
    Pack(PermRef, Tag, Vec<ValueRef>),                                      //Generates a value for a multi ctr mutli field type
    Invoke(PermRef, Vec<ValueRef>),                                         //Invokes a Callable
    TryInvoke(PermRef, Vec<(bool, ValueRef)>, Exp, Exp),                    //Invokes a Callable and handles failures
    InvokeSig(ValueRef, PermRef, Vec<ValueRef>),                            //Invokes a Signature Value
    TryInvokeSig(ValueRef, PermRef, Vec<(bool, ValueRef)>, Exp, Exp),       //Invokes a Signature Value
    Project(TypeRef, ValueRef),                                             //Produces the Image of the input without consuming it
    UnProject(ValueRef),                                                    //Removes a layer from a nested Projection: If the inner value is Primitive
    RollBack(Vec<ValueRef>, Vec<TypeRef>)                                   //Aborts the current Transaction, can consume and produce values to please the type checker
}

