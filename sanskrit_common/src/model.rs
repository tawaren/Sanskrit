use alloc::vec::Vec;
use crate::encoding::*;
use crate::errors::*;
use core::marker::PhantomData;
use core::fmt::Debug;
use alloc::borrow::ToOwned;

pub const HASH_SIZE:usize = 20;

//A Simple 160bit hash
pub type Hash = [u8;HASH_SIZE];

pub fn hash_from_slice(input:&[u8]) -> Hash {
    array_ref!(input, 0, HASH_SIZE).to_owned()
}

//Represents a reference to the nTh element on the stack from the top (Bernouli Index)
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct ValueRef(pub u16);

//Links that point to components in the storage
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum ModuleLink{
    Remote(Hash),
    This(Hash), //Runtime Only (never serialized)
}

impl Serializable for ModuleLink {
    fn serialize(&self, s: &mut Serializer) -> Result<()> {
        match *self {
            ModuleLink::Remote(h) => h.serialize(s)?,
            ModuleLink::This(h) => h.serialize(s)?
        };
        Ok(())
    }
}

impl<'a> Parsable<'a> for ModuleLink {
    fn parse<A: ParserAllocator>(p: &mut Parser, alloc: &'a A) -> Result<ModuleLink> {
        Ok(ModuleLink::Remote(Hash::parse(p,alloc)?))
    }
}

impl ModuleLink {
    pub fn to_hash(&self) -> Hash {
        match *self {
            ModuleLink::Remote(h) => h,
            ModuleLink::This(h) => h,
        }
    }
}

//Most vectors used in sanskrit have a max len of 255, in some places more is needed
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct LargeVec<T>(pub Vec<T>);

//Represents a Identifier for a constructor of a specific Adt
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable, VirtualSize)]
pub struct Tag(pub u8);

//the Parsable Derive is special for the next two see encoding
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Ptr<'a, T>(pub &'a T);

#[derive(Copy, Clone, Eq)]
pub struct SlicePtr<'a, T>(pub u16, pub *const T, pub PhantomData<&'a [T]>);

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct MutSlicePtr<'a, T>(pub u16, pub *mut T, pub PhantomData<&'a [T]>);
