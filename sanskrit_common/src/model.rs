use alloc::vec::Vec;
use crate::encoding::*;
use crate::errors::*;
use core::fmt::Debug;
use alloc::borrow::ToOwned;

pub const HASH_SIZE:usize = 20;

//A Simple 160bit hash
pub type Hash = [u8;HASH_SIZE];

pub fn hash_from_slice(input:&[u8]) -> Hash {
    array_ref!(input, 0, HASH_SIZE).to_owned()
}

//Represents a reference to the nTh element on the stack from the top (Bernouli Index)
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ValueRef(pub u16);

//Links that point to components in the storage
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct ModuleLink(Hash);

impl ModuleLink {
    pub fn new(module_hash:Hash) -> Self {ModuleLink(module_hash)}
    pub fn module_hash(&self) -> &Hash { &self.0 }
}

//Most vectors used in sanskrit have a max len of 255, in some places more is needed
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct LargeVec<T>(pub Vec<T>);

//Represents a Identifier for a constructor of a specific Adt
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug, Parsable, Serializable)]
pub struct Tag(pub u8);
