use errors::*;
use encoding::*;
use model::Capability;

//A special set for representing the capabilities
//it uses an u8 as bitset where each bit i 1 of the 8 capabilities
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug, Parsable, Serializable, VirtualSize)]
pub struct CapSet(pub u8);

//Assosiation between capabilities and bits
//Eac capabulity is assosiated with 1 bit in a u8
const DROP_MASK:u8 = 1;
const COPY_MASK:u8 = 1 << 1;
const PERSIST_MASK:u8 = 1 << 2;
const CONSUME_MASK:u8 = 1 << 3;
const INSPECT_MASK:u8 = 1 << 4;
const EMBED_MASK:u8 = 1 << 5;
const CREATE_MASK:u8 = 1 << 6;

impl CapSet {

    //define some capability sets
    const RECURSIVE_MASK:u8 = DROP_MASK | COPY_MASK | PERSIST_MASK ;

    const NON_RECURSIVE_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | CREATE_MASK ;
    const STD_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | CREATE_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK;

    const TXT_LOCAL_MASK:u8 = CONSUME_MASK | INSPECT_MASK | EMBED_MASK | DROP_MASK | COPY_MASK;

    const LIT_MASK:u8 = EMBED_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK | CREATE_MASK ;

    const OPAQUE_MASK:u8 = EMBED_MASK | DROP_MASK | COPY_MASK | PERSIST_MASK;

    const OPAQUE_AFFINE_MASK:u8 = EMBED_MASK | DROP_MASK  | PERSIST_MASK ;

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
    pub fn from_cap(native: Capability) -> Self {
        CapSet(match native{
            Capability::Drop => DROP_MASK,
            Capability::Copy => COPY_MASK,
            Capability::Persist => PERSIST_MASK,
            Capability::Consume => CONSUME_MASK,
            Capability::Inspect => INSPECT_MASK,
            Capability::Embed => EMBED_MASK,
            Capability::Create => CREATE_MASK,
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

    //No Create,Consume,Inspect
    pub fn lit() -> Self {
        CapSet(CapSet::LIT_MASK)
    }


    //No Create,Consume,Inspect
    pub fn local() -> Self {
        CapSet(CapSet::TXT_LOCAL_MASK)
    }

    //adds a cap to the set
    pub fn with_elem(self, native_cap: Capability) -> Self {
        CapSet(self.0 | CapSet::from_cap(native_cap).0)
    }

    //add multiple caps to the set
    pub fn with_elems<I:Iterator<Item=Capability>>(mut self, iter:I) -> Self {
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
    pub fn contains(self, native_cap: Capability) -> bool {
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
/*
impl NativeType {
    //Assosiates a capset to each basic type
    pub fn base_caps(self) ->  CapSet {
        //get the base caps (does not consider generics)
        match self {
            NativeType::SInt(_)
            | NativeType::UInt(_)
            | NativeType::Data(_)
            | NativeType::PrivateId
            | NativeType::PublicId
            | NativeType::Nothing => CapSet::opaque(),
            NativeType::Bool
            | NativeType::Tuple(_)
            | NativeType::Alternative(_) => CapSet::open(),

        }
    }
}
*/