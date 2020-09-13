use utils::Crc;
use alloc::vec::Vec;
use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use model::linking::Link;
use model::bitsets::{CapSet, BitSet, PermSet};
use model::Permission;

//Global structs and enum to represent language elements
//  they are global as they do no longer depend on a context and can be used cross module

//A Type (Can be compared for type equality)
//Capabilities here do already take into account the generics
#[derive(Debug)]
pub enum ResolvedType {
    //A generic import that was not substituted (happens only if it is a top level generic)
    Generic { caps:CapSet, offset:u8, is_phantom:bool },
    //A Projection type capturing the information of a projected type
    Projection { depth:u8, un_projected: Crc<ResolvedType> },
    //An signature type (can be from same module if Module == This)
    Sig { generic_caps: CapSet, caps: CapSet, module:Crc<ModuleLink>, offset:u8, applies: Vec<Crc<ResolvedType>>},
    //An imported type (can be from same module if Module == This)
    Data { generic_caps: CapSet, caps: CapSet, module:Crc<ModuleLink>, offset:u8, applies: Vec<Crc<ResolvedType>>},
    //An imported type (can be from same module if Module == This)
    Lit { generic_caps: CapSet, caps: CapSet, module:Crc<ModuleLink>, offset:u8, applies: Vec<Crc<ResolvedType>>, size:u16},
    //An External Type
    Virtual(Hash)
}


#[derive(Debug)]
pub enum ResolvedPermission{
    TypeSig{perm:PermSet, typ:Crc<ResolvedType>, signature:Crc<ResolvedSignature>},
    FunSig{perm:PermSet, fun:Crc<ResolvedCallable>, signature:Crc<ResolvedSignature>},
    TypeData{perm:PermSet, typ:Crc<ResolvedType>, ctrs: Crc<Vec<Vec<Crc<ResolvedType>>>>},
    TypeLit{perm:PermSet, typ:Crc<ResolvedType>, size:u16},
}

//A Callable
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ResolvedCallable {
    Function{module:Crc<ModuleLink>, offset:u8, applies: Vec<Crc<ResolvedType>>},
    Implement{module:Crc<ModuleLink>, offset:u8, applies: Vec<Crc<ResolvedType>>},
}

//A function signature (retrieved from applying generics to a function)
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ResolvedSignature {
    pub params:Vec<ResolvedParam>,          //The parameters (including if they are consumed)
    pub returns:Vec<Crc<ResolvedType>>,        //returns including if they borrow something (in bernouli notation)
    pub transactional:bool
}

//Parameters of a Signature
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ResolvedParam {
    pub typ: Crc<ResolvedType>,
    //The typ of the param
    pub consumes: bool,               //is it consumed when the function is applied
}

impl Crc<ResolvedPermission> {

    pub fn check_value_permission(&self, expected_typ:&Crc<ResolvedType>, expected_perm:Permission) -> bool {
        match **self {
            ResolvedPermission::TypeLit { perm, ref typ, ..}
            | ResolvedPermission::TypeData { perm, ref typ, .. }
            | ResolvedPermission::TypeSig { perm, ref typ, .. } => expected_typ == typ && perm.contains(expected_perm),
            _ => false
        }
    }

    pub fn check_permission(&self, expected_perm:Permission) -> bool {
        match **self {
            ResolvedPermission::TypeSig { perm, .. }
            | ResolvedPermission::TypeData { perm, .. }
            | ResolvedPermission::TypeLit { perm, .. }
            | ResolvedPermission::FunSig {perm, ..} => perm.contains(expected_perm),
        }
    }

    pub fn get_type(&self) -> Result<&Crc<ResolvedType>> {
        match **self {
            ResolvedPermission::TypeLit { ref typ, ..}
            | ResolvedPermission::TypeData { ref typ, .. }
            | ResolvedPermission::TypeSig { ref typ, .. } => Ok(typ),
            _ => error(||"Only sig, data & lit permissions have types"),
        }
    }

    pub fn get_fun(&self) -> Result<&Crc<ResolvedCallable>> {
        match **self {
            ResolvedPermission::FunSig { ref fun, .. } => Ok(fun),
            _ => error(||"Only fun permissions have funs"),
        }
    }


    pub fn get_sig(&self) -> Result<&Crc<ResolvedSignature>> {
        match **self {
            ResolvedPermission::TypeSig { ref signature, .. }
            | ResolvedPermission::FunSig { ref signature, .. } => Ok(signature),
            _ => error(||"Only call & implement permissions have signatures"),
        }
    }

    pub fn get_ctrs(&self) -> Result<&Crc<Vec<Vec<Crc<ResolvedType>>>>> {
        match **self {
            ResolvedPermission::TypeData { ref ctrs, .. } => Ok(ctrs),
            _ => error(||"Only create, consume & inspect permissions have constructors"),
        }
    }

    pub fn get_lit_size(&self) -> Result<u16> {
        match **self {
            ResolvedPermission::TypeLit { size, .. } => Ok(size),
            _ => error(||"Only lit create permissions have a size"),
        }
    }

}

//Some usefull functions on the Type
impl Crc<ResolvedType> {
    //Extracts the capabilities
    // This has the recursive caps injected into the generics to ensure that we can build types that are polymorphic in respect to recursive capabilities
    //   Thanks to this we can have: <Copy,Drop,Persist,...> Option[<Embed> T]{None;Some(T);} and use it safely with Copy, Drop & Persist types (the Option will loose Copy, Drop, Persist if T does not have it)
    //   Without this: we would need: a CopyOption, DropOption, PersistOption, CopyDropOption, ..... , CopyDropPersistOption
    //  This must be used when checking adt fields against the adt base caps
    // Note: this only influences generics and applied types with generic inputs
    pub fn get_generic_caps(&self) -> CapSet {
        match **self {
            ResolvedType::Sig { generic_caps, .. }
            | ResolvedType::Lit { generic_caps, .. }
            | ResolvedType::Data { generic_caps, .. } => generic_caps,
            ResolvedType::Generic { .. }
            | ResolvedType::Projection { .. }  => CapSet::all(),
            ResolvedType::Virtual(_) => CapSet::empty()
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
            | ResolvedType::Sig { caps, .. }
            | ResolvedType::Lit { caps, .. }
            | ResolvedType::Data { caps, .. } => caps,
            ResolvedType::Projection { .. } => CapSet::all(),
            ResolvedType::Virtual(_) => CapSet::empty()
        }
    }

    pub fn get_target(&self) -> &Crc<ResolvedType> {
        match **self {
            ResolvedType::Projection { ref un_projected, .. } => {
                assert!(if let ResolvedType::Projection{..} = **un_projected {false} else {true});
                un_projected
            },
            _ => self
        }
    }

    pub fn get_projection_depth(&self) -> u8 {
        match **self {
            ResolvedType::Projection { depth, .. } => depth,
            _ => 0
        }
    }

    //checks if this type is a literal
    pub fn is_literal(&self) -> bool {
        match **self {
            ResolvedType::Lit {  .. }  => true,
            _ => false
        }
    }

    //checks if this type is local (from current module)
    pub fn is_local(&self) -> bool {
        match **self {
            ResolvedType::Sig { ref module, .. }
            | ResolvedType::Lit { ref module, .. }
            | ResolvedType::Data { ref module, .. } => module.is_local_link(),
            ResolvedType::Virtual(_)
            | ResolvedType::Generic { .. }
            | ResolvedType::Projection {  .. } => false,
        }
    }
}


//Some usefull functions on the Callable
impl Crc<ResolvedCallable> {

    //checks if this type is local (from current module)
    pub fn is_local(&self) -> bool {
        match **self {
            ResolvedCallable::Function { ref module, .. }
            | ResolvedCallable::Implement { ref module, .. } => module.is_local_link(),
        }
    }
}
