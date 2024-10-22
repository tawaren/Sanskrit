use alloc::vec::Vec;
use sanskrit_common::model::*;
use sp1_zkvm_col::arena::URef;
use crate::loader::ResolvedCtrs;
use crate::model::linking::FastModuleLink;
use crate::model::bitsets::{CapSet, BitSet, PermSet};
use crate::model::Permission;

//Global structs and enum to represent language elements
//  they are global as they do no longer depend on a context and can be used cross module


#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ResolvedComponent {
    pub module:FastModuleLink,
    pub offset:u8,
    pub applies:Vec<URef<'static,ResolvedType>>
}

//A Type (Can be compared for type equality)
//Capabilities here do already take into account the generics
#[derive(Debug)]
pub enum ResolvedType {
    //A generic import that was not substituted (happens only if it is a top level generic)
    Generic { caps:CapSet, offset:u8, is_phantom:bool },
    //A Projection type capturing the information of a projected type
    Projection { depth:u8, un_projected: URef<'static,ResolvedType> },
    //An signature type
    // As a signature ignores its generics when computing its caps generic_caps == caps == base_caps
    Sig { caps: CapSet, base: ResolvedComponent },
    //An imported type
    Data { generic_caps: CapSet, caps: CapSet, base: ResolvedComponent },
    //An imported type
    Lit { generic_caps: CapSet, caps: CapSet, base: ResolvedComponent, size:u16},
    //An External Type
    Virtual(Hash)
}


#[derive(Debug)]
pub enum ResolvedPermission{
    TypeSig{perm:PermSet, typ: URef<'static,ResolvedType>, signature: URef<'static,ResolvedSignature>},
    FunSig{perm:PermSet, fun: URef<'static,ResolvedCallable>, signature: URef<'static,ResolvedSignature>},
    TypeData{perm:PermSet, typ: URef<'static,ResolvedType>, ctrs: URef<'static,ResolvedCtrs>},
    TypeLit{perm:PermSet, typ: URef<'static,ResolvedType>, size:u16},
}

//A Callable
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ResolvedCallable {
    Function{base:ResolvedComponent},
    Implement{base:ResolvedComponent},
}

//A function signature (retrieved from applying generics to a function)
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ResolvedSignature {
    pub params:Vec<ResolvedParam>,          //The parameters (including if they are consumed)
    pub returns:Vec<URef<'static,ResolvedType>>,     //returns including if they borrow something (in bernouli notation)
    pub transactional:bool
}

//Parameters of a Signature
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ResolvedParam {
    pub typ: URef<'static,ResolvedType>,
    //The typ of the param
    pub consumes: bool,               //is it consumed when the function is applied
}

impl ResolvedPermission {

    pub fn check_value_permission(&self, expected_typ:URef<'static,ResolvedType>, expected_perm:Permission) -> bool {
        match *self {
            ResolvedPermission::TypeLit { perm, typ, ..}
            | ResolvedPermission::TypeData { perm, typ, .. }
            | ResolvedPermission::TypeSig { perm, typ, .. } => expected_typ == typ && perm.contains(expected_perm),
            _ => false
        }
    }

    pub fn check_permission(&self, expected_perm:Permission) -> bool {
        match *self {
            ResolvedPermission::TypeSig { perm, .. }
            | ResolvedPermission::TypeData { perm, .. }
            | ResolvedPermission::TypeLit { perm, .. }
            | ResolvedPermission::FunSig {perm, ..} => perm.contains(expected_perm),
        }
    }

    pub fn get_type(&self) -> URef<'static,ResolvedType> {
        match *self {
            ResolvedPermission::TypeLit { typ, ..}
            | ResolvedPermission::TypeData { typ, .. }
            | ResolvedPermission::TypeSig { typ, .. } => typ,
            _ => panic!("Only sig, data & lit permissions have types"),
        }
    }

    pub fn get_fun(&self) -> URef<'static,ResolvedCallable> {
        match *self {
            ResolvedPermission::FunSig { fun, .. } => fun,
            _ => panic!("Only fun permissions have funs"),
        }
    }


    pub fn get_sig(&self) -> URef<'static,ResolvedSignature> {
        match *self {
            ResolvedPermission::TypeSig { signature, .. }
            | ResolvedPermission::FunSig { signature, .. } => signature,
            _ => panic!("Only call & implement permissions have signatures"),
        }
    }

    pub fn get_ctrs(&self) -> URef<'static,ResolvedCtrs> {
        match *self {
            ResolvedPermission::TypeData { ctrs, .. } => ctrs,
            _ => panic!("Only create, consume & inspect permissions have constructors"),
        }
    }

    pub fn get_lit_size(&self) -> u16 {
        match *self {
            ResolvedPermission::TypeLit { size, .. } => size,
            _ => panic!("Only lit create permissions have a size"),
        }
    }

}

//Some usefull functions on the Type
impl ResolvedType {
    //Extracts the capabilities
    // This has the recursive caps injected into the generics to ensure that we can build types that are polymorphic in respect to recursive capabilities
    //   Thanks to this we can have: <Copy,Drop,Persist,...> Option[<Embed> T]{None;Some(T);} and use it safely with Copy, Drop & Persist types (the Option will loose Copy, Drop, Persist if T does not have it)
    //   Without this: we would need: a CopyOption, DropOption, PersistOption, CopyDropOption, ..... , CopyDropPersistOption
    //  This must be used when checking adt fields against the adt base caps
    // Note: this only influences generics and applied types with generic inputs
    pub fn get_generic_caps(&self) -> CapSet {
        match *self {
            ResolvedType::Sig { caps:generic_caps, .. } //For Sigs: generic_caps == caps (all sigs ignore generics)
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
        match *self {
            ResolvedType::Generic { caps, .. }
            | ResolvedType::Sig { caps, .. }
            | ResolvedType::Lit { caps, .. }
            | ResolvedType::Data { caps, .. } => caps,
            ResolvedType::Projection { .. } => CapSet::all(),
            ResolvedType::Virtual(_) => CapSet::empty()
        }
    }

    pub fn get_target(&self) -> &ResolvedType {
        match *self {
            ResolvedType::Projection { ref un_projected, .. } => {
                assert!(if let ResolvedType::Projection{..} = **un_projected {false} else {true});
                &un_projected
            },
            _ => self
        }
    }

    pub fn get_projection_depth(&self) -> u8 {
        match *self {
            ResolvedType::Projection { depth, .. } => depth,
            _ => 0
        }
    }

    //checks if this type is a literal
    pub fn is_literal(&self) -> bool {
        match *self {
            ResolvedType::Lit {  .. }  => true,
            _ => false
        }
    }

    //checks if this type is a literal
    pub fn is_data(&self) -> bool {
        match *self {
            ResolvedType::Data {  .. }  => true,
            _ => false
        }
    }

    pub fn is_defining_module(&self, target:&FastModuleLink) -> bool {
        match *self {
            ResolvedType::Sig { ref base, .. }
            | ResolvedType::Lit { ref base, .. }
            | ResolvedType::Data { ref base, .. } => &base.module == target,
            ResolvedType::Virtual(_)
            | ResolvedType::Generic { .. }
            | ResolvedType::Projection {  .. } => false,
        }
    }
}

pub fn get_target(typ:URef<'static,ResolvedType>) -> URef<'static,ResolvedType> {
    match *typ {
        ResolvedType::Projection { un_projected, .. } => {
            assert!(if let ResolvedType::Projection{..} = *un_projected {false} else {true});
            un_projected
        },
        _ => typ
    }
}
