use sanskrit_common::encoding::*;
use crate::model::{Capability, Permission};

pub trait BitEntry {
    fn to_mask(&self) -> u8;
}

pub trait BitSet<T:BitEntry> where Self:Sized {
    fn to_bits(&self) -> u8;
    fn from_bits(idx:u8) -> Self;

    //generate a empty set
    fn empty() -> Self {
        Self::from_bits(0)
    }

    //A set with a single capability
    fn from_entry(entry: T) -> Self { Self::from_bits(entry.to_mask())}

    //adds a cap to the set
    fn with_elem(self, elem: T) -> Self {
        Self::from_bits(self.to_bits() | elem.to_mask())
    }

    //add multiple caps to the set
    fn with_elems<I:Iterator<Item=T>>(mut self, iter:I) -> Self {
        for add in iter {
            self = self.with_elem(add)
        }
        self
    }

    //checks if it is the empty set
    fn is_empty(self) -> bool {
        self.to_bits() == 0
    }

    //checks if a cap is contained
    fn contains(self, entry: T) -> bool {
        self.to_bits() & entry.to_mask() != 0
    }

    //merges two cap sets, the result containing all elems from both sets
    fn union(self, set2:Self) -> Self {
        Self::from_bits(self.to_bits() | set2.to_bits())
    }

    //checks if set2 has all the elems of self in it
    fn is_subset_of(self, set2:Self) -> bool {
        self.to_bits() & set2.to_bits() == self.to_bits()
    }

    //removes the element in set2 from self
    fn without(self, set2:Self) -> Self {
        Self::from_bits(self.to_bits() & !set2.to_bits())
    }

    //returns the elements that are not in both sets
    fn difference(self, set2:Self) -> Self{
        Self::from_bits((self.to_bits() & set2.to_bits()) ^ (self.to_bits() | set2.to_bits()))
    }

    //returns the caps that are in both sets
    fn intersect(self, set2:Self) -> Self{
        Self::from_bits(self.to_bits() & set2.to_bits())
    }
}

//A special set for representing the capabilities
//it uses an u8 as bitset where each bit i 1 of the 8 capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable)]
pub struct CapSet(pub u8);

//Assosiation between capabilities and bits
//Eac capabulity is assosiated with 1 bit in a u8
const DROP_MASK:u8 = 1;
const COPY_MASK:u8 = 1 << 1;
const PERSIST_MASK:u8 = 1 << 2;
const PRIMITIVE_MASK:u8 = 1 << 3;
const VALUE_MASK:u8 = 1 << 4;
const UNBOUND_MASK:u8 = 1 << 5;

impl BitSet<Capability> for CapSet {
    fn to_bits(&self) -> u8 {
        self.0
    }

    fn from_bits(elems: u8) -> Self {
        CapSet(elems)
    }
}

impl BitEntry for Capability {
    fn to_mask(&self) -> u8 {
        match *self {
            Capability::Drop => DROP_MASK,
            Capability::Copy => COPY_MASK,
            Capability::Persist => PERSIST_MASK,
            Capability::Primitive => PRIMITIVE_MASK,
            Capability::Value => VALUE_MASK,
            Capability::Unbound => UNBOUND_MASK,
        }
    }
}

impl CapSet {

    const ALL_MASK:u8 = VALUE_MASK | UNBOUND_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK | PRIMITIVE_MASK ;

    const NOT_SIG_MASK:u8 = VALUE_MASK | UNBOUND_MASK | COPY_MASK | PERSIST_MASK | PRIMITIVE_MASK;

    const OPAQUE_AFFINE_MASK:u8 =  DROP_MASK  | PERSIST_MASK ;

    const PRIMITIVE_IMPLICATIONS: u8 = DROP_MASK | COPY_MASK | PERSIST_MASK | VALUE_MASK | UNBOUND_MASK ;

    pub fn check_constraints(self){
        if self.contains(Capability::Primitive) && !CapSet(CapSet::PRIMITIVE_IMPLICATIONS).is_subset_of(self) {
            panic!("Types with the Primitive capability must have Drop, Copy, Persist, and Unbound as well")
        }
        /* Todo: is this usefull if yes comment in (and change a lot of tests)
        if self.contains(Capability::Persist) && !self.contains(Capability::Unbound) {
            return error(||"Types with the Persist capability must have Unbound as well")
        }*/
    }

    //generate a empty set
    pub fn all() -> Self {
        CapSet(Self::ALL_MASK)
    }

    //standard capability set (same as languages without caps)
    pub fn primitive() -> Self {
        CapSet(CapSet::ALL_MASK)
    }

    //signature capability set
    pub fn signature_prohibited() -> Self  {
        CapSet(CapSet::NOT_SIG_MASK)
    }

    //No Create,Consume,Inspect & No Copy
    pub fn opaque_affine() -> Self {
        CapSet(CapSet::OPAQUE_AFFINE_MASK)
    }
}


//A special set for representing the capabilities
//it uses an u8 as bitset where each bit i 1 of the 8 capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable)]
pub struct PermSet(pub u8);

//Assosiation between permissions and bits
//Eac permission is assosiated with 1 bit in a u8
const CREATE_MASK:u8 = 1;
const CONSUME_MASK:u8 = 1 << 1;
const INSPECT_MASK:u8 = 1 << 2;
const CALL_MASK:u8 = 1 << 3;
const IMPLEMENT_MASK:u8 = 1 << 4;

impl BitSet<Permission> for PermSet {
    fn to_bits(&self) -> u8 {
        self.0
    }

    fn from_bits(elems: u8) -> Self {
        PermSet(elems)
    }
}

impl BitEntry for Permission {
    fn to_mask(&self) -> u8 {
        match *self {
            Permission::Create => CREATE_MASK,
            Permission::Consume => CONSUME_MASK,
            Permission::Inspect => INSPECT_MASK,
            Permission::Call => CALL_MASK,
            Permission::Implement => IMPLEMENT_MASK,
        }
    }
}

impl PermSet {

    const ALL_MASK:u8 = CREATE_MASK | CONSUME_MASK | INSPECT_MASK | CALL_MASK | IMPLEMENT_MASK ;

    const DATA_MASK:u8 = CREATE_MASK | CONSUME_MASK | INSPECT_MASK;

    const SIG_MASK:u8 = CALL_MASK | IMPLEMENT_MASK;

    const CALLABLE_MASK:u8 = CALL_MASK ;

    const LIT_MASK:u8 = CREATE_MASK;

    //generate a empty set
    pub fn all() -> Self {
        PermSet(Self::ALL_MASK)
    }

    pub fn callable_perms() -> Self {
        PermSet(Self::CALLABLE_MASK)
    }

    pub fn data_perms() -> Self {
        PermSet(Self::DATA_MASK)
    }

    pub fn sig_perms() -> Self {
        PermSet(Self::SIG_MASK)
    }

    pub fn lit_perms() -> Self {
        PermSet(Self::LIT_MASK)
    }
}
