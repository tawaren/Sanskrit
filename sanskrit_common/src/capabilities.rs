use errors::*;
use encoding::*;
use model::NativeCap;
use model::NativeType;

//A special set for representing the capabilities
//it uses an u8 as bitset where each bit i 1 of the 8 capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
pub struct CapSet(pub u8);

//Assosiation between capabilities and bits
//Eac capabulity is assosiated with 1 bit in a u8
const DROP_MASK:u8 = 1 << 0;
const COPY_MASK:u8 = 1 << 1;
const PERSIST_MASK:u8 = 1 << 2;
const CONSUME_MASK:u8 = 1 << 3;
const INSPECT_MASK:u8 = 1 << 4;
const EMBED_MASK:u8 = 1 << 5;
const CREATE_MASK:u8 = 1 << 6;
const INDEXED_MASK:u8 = 1 << 7;

impl CapSet {

    //define some capability sets
    const RECURSIVE_MASK:u8 = DROP_MASK | COPY_MASK | PERSIST_MASK;

    const NON_RECURSIVE_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | CREATE_MASK | INDEXED_MASK;

    const STD_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | CREATE_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK;

    const OPAQUE_MASK:u8 = EMBED_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK;

    const OPAQUE_AFFINE_MASK:u8 = EMBED_MASK | DROP_MASK  | PERSIST_MASK;

    const INDEXED_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | CREATE_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK | INDEXED_MASK;

    //generate a empty set
    pub fn empty() -> Self {
        CapSet(0)
    }

    //all the non recursive ones
    pub fn non_recursive() -> Self {
        CapSet(Self::NON_RECURSIVE_MASK)
    }

    //all the recursive ones
    pub fn recursive() -> Self {
        CapSet(Self::RECURSIVE_MASK)
    }

    //A set with a single capability
    pub fn from_cap(native:NativeCap) -> Self {
        CapSet(match native{
            NativeCap::Drop => DROP_MASK,
            NativeCap::Copy => COPY_MASK,
            NativeCap::Persist => PERSIST_MASK,
            NativeCap::Consume => CONSUME_MASK,
            NativeCap::Inspect => INSPECT_MASK,
            NativeCap::Embed => EMBED_MASK,
            NativeCap::Create => CREATE_MASK,
            NativeCap::Indexed => INDEXED_MASK,
        })
    }

    //standard capability set (same as languages without caps)
    pub fn open() -> Self {
        CapSet(CapSet::STD_MASK)
    }

    //No Create,Consume,Inspect & No Copy
    pub fn opaque_affine() -> Self {
        CapSet(CapSet::OPAQUE_AFFINE_MASK)
    }

    //This has everithing
    pub fn indexed() -> Self {
        CapSet(CapSet::INDEXED_MASK)
    }

    //No Create,Consume,Inspect
    pub fn opaque() -> Self {
        CapSet(CapSet::OPAQUE_MASK)
    }

    //adds a cap to the set
    pub fn with_elem(self, native_cap:NativeCap) -> Self {
        CapSet(self.0 | CapSet::from_cap(native_cap).0)
    }

    //add multiple caps to the set
    pub fn with_elems<I:Iterator<Item=NativeCap>>(mut self, iter:I) -> Self {
        for add in iter {
            self = self.with_elem(add)
        }
        self
    }

    //checks if it is the empty set
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    //checks if a cap is contained
    pub fn contains(self, native_cap:NativeCap) -> bool {
        self.0 & CapSet::from_cap(native_cap).0 != 0
    }

    //merges two cap sets, the result containing all elems from both sets
    pub fn union(self, set2:Self) -> Self {
        CapSet(self.0 | set2.0)
    }

    //checks if set2 has all the elems of self in it
    pub fn is_subset_of(self, set2:Self) -> bool {
        self.0 & set2.0 == self.0
    }

    //removes the element in set2 from self
    pub fn without(self, set2:Self) -> CapSet {
        CapSet(self.0 & !set2.0)
    }

    //returns the elements that are not in both sets
    pub fn difference(self, set2:Self) -> Self{
        CapSet((self.0 & set2.0) ^ (self.0 | set2.0))
    }

    //returns the caps that are in both sets
    pub fn intersect(self, set2:Self) -> CapSet{
        CapSet(self.0 & set2.0)
    }
}

impl NativeType {
    //Assosiates a capset to each basic type
    pub fn base_caps(self) ->  CapSet {
        //get the base caps (does not consider generics)
        match self {
            NativeType::SInt(_)
            | NativeType::UInt(_)
            | NativeType::Data(_)
            | NativeType::Ref => CapSet::opaque(),
            NativeType::Bool
            | NativeType::Tuple(_)
            | NativeType::Alternative(_) => CapSet::open(),
            NativeType::Context => CapSet::from_cap(NativeCap::Drop),
            NativeType::Unique
            | NativeType::Singleton => CapSet::opaque_affine(),
            NativeType::Index => CapSet::indexed(),
        }
    }
}