use alloc::rc::Rc;
use utils::Crc;
use sanskrit_common::capabilities::*;
use alloc::collections::BTreeSet;
use sanskrit_common::model::ValueRef;
use native::base::is_native_literal;
use alloc::prelude::*;
use sanskrit_common::model::*;


//Global structs and enum to represent language elements
//  they are global as they do no longer depend on a context and can be used cross module

//An error
#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum ResolvedErr {
    Import { offset:u8,  module:Rc<ModuleLink>},        //Module and offset of the imported erreor
    Native { err: NativeError, }                        //Specific Native Error
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
//needed to capture phantom information which influence the Capability calculation
pub struct ResolvedApply{
    pub is_phantom:bool,
    pub typ:Crc<ResolvedType>
}

//A Type (Can be compared for type equality)
//Capabilities here do already take into account the generics
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum ResolvedType {
    //A generic import that was not substituted (happens only if it is a top level generic)
    Generic { extended_caps: CapSet, caps:CapSet, offset:u8, is_phantom:bool },
    //An Image type capturing the information of another type
    Image { typ: Crc<ResolvedType> },
    //An imported type (can be from same module if Module == This)
    Import { extended_caps: CapSet, caps: CapSet, base_caps:CapSet, module:Rc<ModuleLink> , offset:u8, applies: Vec<ResolvedApply>},
    //A Native type
    Native { extended_caps: CapSet, caps: CapSet, base_caps:CapSet, typ: NativeType, applies:Vec<ResolvedApply>}
}

//A function
pub enum ResolvedFunction {
    //An imported function (can be from same module if Module == This)
    Import { module:Rc<ModuleLink> , offset:u8, applies: Vec<Crc<ResolvedType>> },
    //A Native function
    Native { typ: NativeFunc, applies:Vec<Crc<ResolvedType>> }
}

//A function signature (retrieved from applying generics to a function)
pub struct ResolvedSignature {
    pub params:Vec<ResolvedParam>,          //The parameters (including if they are consumed)
    pub returns:Vec<ResolvedReturn>,        //returns including if they borrow something (in bernouli notation)
    pub risks:BTreeSet<Rc<ResolvedErr>>,    //the risks that the function can throw
}

//Parameters of a Signature
pub struct ResolvedParam {
    pub typ: Crc<ResolvedType>,
    //The typ of the param
    pub consumes: bool,               //is it consumed when the function is applied
}

//Returns of a Function
pub struct ResolvedReturn {
    pub typ:Crc<ResolvedType>,      //The type of the return
    pub borrows:Vec<ValueRef>       //an indicator if the return borrows something
}

//Some usefull functions on the Type
impl Crc<ResolvedType> {
    //Extracts the capabilities
    // This has the recursive caps injected into the generics to ensure that we can build types that are polymorphic in respect to recursive capabilities
    //   Thanks to this we can have: <Copy,Drop,Persist,...> Option[<Embed> T]{None;Some(T);} and use it safely with Copy, Drop & Persist types (the Option will loose Copy, Drop, Persist if T does not have it)
    //   Without this: we would need: a CopyOption, DropOption, PersistOption, CopyDropOption, ..... , CopyDropPersistOption
    //  This must be used when checking adt fields against the adt base caps
    // Note: this only influences generics and applied types with generic inputs
    pub fn get_extended_caps(&self) -> CapSet {
        match **self {
            ResolvedType::Generic { extended_caps, .. }
            | ResolvedType::Native { extended_caps, .. }
            | ResolvedType::Import { extended_caps, .. } => extended_caps,
            ResolvedType::Image { .. } => CapSet::open(),
        }
    }

    //Extracts the capabilities
    // This has nothing injected and have the constraints only in case of generics
    //   Thanks to this we can still have a Option[<Drop,Embed> T] if we want to deny using it with non-drop types.
    //  This must be used when checking that a type applied to a generic full fills its constraint
    //  This must be used when checking if the correct caps are available to execute a operation
    // Note: this only influences generics and applied types with generic inputs
    pub fn get_caps(&self) -> CapSet {
        match **self {
            ResolvedType::Generic { caps, .. }
            | ResolvedType::Native { caps, .. }
            | ResolvedType::Import { caps, .. } => caps,
            ResolvedType::Image { .. } => CapSet::open()
        }
    }

    //checks if this type is a literal
    pub fn is_literal(&self) -> bool {
        match **self {
            ResolvedType::Image { .. } | ResolvedType::Generic { .. } | ResolvedType::Import { .. } => false,
            ResolvedType::Native { typ, .. } => is_native_literal(typ),
        }
    }

    //checks if this type is local (from current module)
    pub fn is_local(&self) -> bool {
        match **self {
            ResolvedType::Import { ref module, .. } => match **module {
                ModuleLink::This(_) => true,
                _ => false
            },
            ResolvedType::Image { .. } |  ResolvedType::Native { .. } | ResolvedType::Generic { .. }  => false,
        }
    }
}
