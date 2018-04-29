

pub const HASH_SIZE:usize = 20;
#[derive(Copy, Clone, Debug)]
pub struct Hash<'a>{
    pub d:&'a [u8]
}

pub const MAGIC_NUMBER_SIZE: usize = 1;

pub const MEMBER_INDEX_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct MemberIndex(pub u8);

pub const FLAG_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Flag(pub bool);

pub const VERSION_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Version(pub u8);

pub const COEFFICIENT_SIZE: usize = 2;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Coefficient(pub u16);

pub const FIELD_SIZE: usize = 2;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Field(pub Control, pub TypeId);

pub const PTR_SIZE: usize = 2;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Ptr(pub u16);

pub const LENGTH_SIZE: usize = 2;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Length(pub u16);

pub const TAG_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Tag(pub u8);

pub const MODULE_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ModuleId(pub u8);

pub const TYPE_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct TypeId(pub u8);

pub const VALUE_ID_SIZE: usize = 2;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ValueId(pub u16);

pub const FUN_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct FunId(pub u8);

pub const CTR_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct CtrId(pub u8);

pub const INIT_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct InitId(pub u8);

pub const PARAM_ID_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ParamId(pub u8);

pub const AMOUNT_SIZE: usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Amount(pub u8);

pub const PRIVILEGES_SIZE:usize = 2;
pub const UNWRAP_PRIVILEGE:u16 = 1 << 0 | PERSIST_PRIVILEGE;
pub const WRAP_PRIVILEGE:u16 = 1 << 1 | PERSIST_PRIVILEGE;
pub const ACCESS_PRIVILEGE:u16 = 1 << 2;
pub const CREATE_PRIVILEGE:u16 = 1 << 3;
pub const LOAD_PRIVILEGE:u16 = 1 << 4 | PERSIST_PRIVILEGE;
pub const WRITE_PRIVILEGE:u16 = 1 << 5 | PERSIST_PRIVILEGE;
pub const DISCARD_PRIVILEGE:u16 = 1 << 6;
pub const COPY_PRIVILEGE:u16 = 1 << 7;
pub const PERSIST_PRIVILEGE:u16 = 1 << 8;
pub const EXTRACT_PRIVILEGE:u16 = 1 << 9 | ACCESS_PRIVILEGE;
pub const ASSEMBLE_PRIVILEGE:u16 = 1 << 10 | CREATE_PRIVILEGE;
pub const ALL_PRIVILEGES:u16 = (1 << 11) -1;

pub const NATIVE_PRIVILEGE:u16 = ASSEMBLE_PRIVILEGE | EXTRACT_PRIVILEGE | DISCARD_PRIVILEGE | COPY_PRIVILEGE | PERSIST_PRIVILEGE;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Privileges(pub u16);

//Annotates function Parameters and Returns as well as ADT params
//Any parameter is one of these
pub const CONTROL_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Control {
    Ref,             //Overwrites behaviour act as Plain but no unpack, just inspect
    Owned,           //Follow the Behaviour
    Borrowed,        //Not allowed on  (inkl let & case of) or Adt: Overwrites behaviour (can not be stored in ADT) else like Plain
    //These three have the role of an optimisation: if a param is never accessed a HHL can mark it unused, which then leads to near zero runtime overhead
    UnusedRef,      //Ref + No interaction that need actual data       (no load, no write, no eq, ...)
    UnusedOwned,    //Owned + No interaction that need actual data     (no inspect, no unpack, no eq, ...)
    UnusedBorrowed, //Borrowed + No interaction that need actual data  (no inspect, no unpack, no eq, ...)

}
//Todo: should controll has 2 fields?? 1 Use, Unused, & 1 Ref, Borrowed, Owned



pub const BOUND_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Bound {
    Dynamic,                //Behaviour bound of this depends on behaviour of specified for result (allows making it stricter)
    Phantom,                //Behaviour will take all but not contribute to end type -- Treats the generic param as if it has phantom behaviour
}                           // Like Behaviour means that a value never can be observed in this adt, unlike on a function it measn if the adt contains a phantom value itself must be a phantom (unobservable)

pub const VISIBILITY_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Visibility {
    Public,
    Private,
    //External, Will be replaced by entrypoints
}

pub const TYPE_KIND_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TypeKind {
    Cell,
    View,
    Normal,
}


pub const EXECUTION_MODE_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ExecutionMode {
    Pure,
    Init,
    Dependent,
    Active,
}

pub const OPTIMIZATION_DECLARATION_SIZE:usize = 1;
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum OptimizationDeclaration {
    Normal,
    Empty,       //Only for optimisation
    Wrapper,     //For Optimisation
}


//OTPIM MEANING ADT:
// EMPTY    => Optim: Value can be generated from type (only one value) -- (value has 0 bits)
// WRAPPER  => Optim: Created instance has same runtime representation as its argument -- (X(a) == a @ runtime) -- (just type differs)

//OTPIM MEANING Function:
// EMPTY    => Optim: Result is Empty and has no sideeffects (result can be generated from type)
// WRAPPER  => Optim: Result has same runtime representation as the functions argument and no sideeffects
