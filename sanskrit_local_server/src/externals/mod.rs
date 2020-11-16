use std::sync::Mutex;
use sanskrit_common::errors::*;
use sanskrit_common::model::{ModuleLink, SlicePtr, ValueRef, Hash, Ptr};
use sanskrit_common::arena::{HeapArena, VirtualHeapArena};
use sanskrit_interpreter::model::{ValueSchema, Kind, Entry, Adt, RuntimeType};
use std::collections::BTreeMap;
use sanskrit_runtime::system::SystemContext;
use std::cell::Cell;
use sanskrit_common::hashing::HashingDomain;
use sanskrit_sled_store::SledStore;
use sanskrit_runtime::direct_stored::{StatefulEntryStoreVerifier, SystemDataManager, StatefulEntryStoreExecutor};
use sanskrit_common::encoding::{VirtualSize, ParserAllocator, Parser};
use sanskrit_runtime::model::{BundleWithHash, BaseTransactionBundle};
use sanskrit_runtime::CONFIG;
use externals::crypto::{join_hash, plain_hash, ecdsa_verify};
use sanskrit_compile::externals::{CompilationExternals, CompilationResult};
use sanskrit_interpreter::externals::{RuntimeExternals, ExecutionInterface};

pub mod i8;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i128;
pub mod u8;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u128;
pub mod data;
pub mod ids;
pub mod eddsa;
pub mod _unsafe;
pub mod crypto;

pub trait External:Sync{
    fn compile_lit<'b,'h>(&self, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(&self, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
    fn compile_call<'b,'h>(&self, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
}

lazy_static! {
    pub static ref EXT_MAP: Mutex<BTreeMap<Hash, &'static dyn External>> = Mutex::new(BTreeMap::new());
}

lazy_static! {
    pub static ref SYS_HASH: Mutex<Cell<Hash>> = Mutex::new(Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
}

lazy_static! {
    pub static ref EDDSA_HASH: Mutex<Cell<Hash>> = Mutex::new(Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
}

pub fn get_ed_dsa_module() -> Hash {EDDSA_HASH.lock().unwrap().get()}


lazy_static! {
    pub static ref SYS_MODS: [fn(Hash)->();16] = [
            |h|{EXT_MAP.lock().unwrap().insert(h,i8::EXT_I8);},          //0
            |h|{EXT_MAP.lock().unwrap().insert(h,i16::EXT_I16);},        //1
            |h|{EXT_MAP.lock().unwrap().insert(h,i32::EXT_I32);},        //2
            |h|{EXT_MAP.lock().unwrap().insert(h,i64::EXT_I64);},        //3
            |h|{EXT_MAP.lock().unwrap().insert(h,i128::EXT_I128);},      //4
            |h|{EXT_MAP.lock().unwrap().insert(h,u8::EXT_U8);},          //5
            |h|{EXT_MAP.lock().unwrap().insert(h,u16::EXT_U16);},        //6
            |h|{EXT_MAP.lock().unwrap().insert(h,u32::EXT_U32);},        //7
            |h|{EXT_MAP.lock().unwrap().insert(h,u64::EXT_U64);},        //8
            |h|{EXT_MAP.lock().unwrap().insert(h,u128::EXT_U128);},      //9
            |h|{EXT_MAP.lock().unwrap().insert(h,data::EXT_DATA);},      //10
            |h|{EXT_MAP.lock().unwrap().insert(h,ids::EXT_IDS);},        //11
            |h|{SYS_HASH.lock().unwrap().set(h);},                       //12
            |h|{EXT_MAP.lock().unwrap().insert(h,eddsa::EXT_ECDSA);},
            |h|{EXT_MAP.lock().unwrap().insert(h,_unsafe::EXT_UNSAFE);}, //14
            |h|{EDDSA_HASH.lock().unwrap().set(h);},                     //15
    ];
}



pub struct ServerExternals;
impl CompilationExternals for ServerExternals {
    fn compile_call<'b, 'h>(module: &ModuleLink, fun_idx: u8, params: SlicePtr<'b, ValueRef>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(|| "Implementation for external module is missing"),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller, alloc)
        }
    }

    fn compile_lit<'b, 'h>(module: &ModuleLink, data_idx: u8, data: SlicePtr<'b, u8>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(|| "Implementation for external module is missing"),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller, alloc)
        }
    }

    fn get_literal_checker<'b, 'h>(module: &ModuleLink, data_idx: u8, len: u16, alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(|| "Implementation for external module is missing"),
            Some(ref imp) => imp.get_literal_checker(data_idx, len, alloc)
        }
    }
}

impl RuntimeExternals for ServerExternals {

    fn typed_system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Hash
            0 => plain_hash(interface, kind, values[0], tail),
            _ => unreachable!()
        }
    }

    fn system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Derive
            0 => join_hash(interface, values[0], values[1], HashingDomain::Derive, tail),
            //EcDsaVerify
            1 => ecdsa_verify(interface, values[0], values[1], values[2], tail),
            _ => unreachable!()
        }
    }
}

pub struct ServerSystemDataManager;
impl<'c> SystemDataManager<BundleWithHash<'c>> for ServerSystemDataManager {

    fn providable_size(typ: Ptr<RuntimeType>) -> Result<u32> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                Ok((Hash::SIZE + 4*Entry::SIZE) as u32)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                Ok((Hash::SIZE + 2*Entry::SIZE) as u32)
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }

    fn providable_gas(typ: Ptr<RuntimeType>) -> Result<u64> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (6 as u64);
                Ok(hash_alloc + pack)
            }
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (6 as u64);
                let hash_cost = 65;
                Ok(hash_alloc + pack + hash_cost)
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }


    fn is_chain_value(typ: Ptr<RuntimeType>) -> bool {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 0 => true,
            _ => false
        }
    }

    //This means we can only provide 1 value per Txt
    fn provided_value_key(typ: Ptr<RuntimeType>, section_no:u8,  txt_no:u8) -> Option<Vec<u8>> {
        match *typ {
            //This means we can only provide 1 value per Txt
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => Some(vec![section_no,txt_no]),
            //For the rest (TxData we can provide as many copies as we want)
            _ => None
        }
    }

    fn create_provided_value<'a, 'h>(bundle: &BundleWithHash, typ: Ptr<RuntimeType>, alloc: &'a VirtualHeapArena<'h>, block_no: u64, section_no:u8,  txt_no:u8) -> Result<Entry<'a>> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
                    Entry {data: alloc.copy_alloc_slice(&bundle.bundle_hash)?},
                    Entry {u64: block_no},
                    Entry {u8: section_no},
                    Entry {u8: txt_no},
                ])?)})
            },
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 2 => {
                let mut context = HashingDomain::Derive.get_domain_hasher();
                //fill the hash with bunlde hash value
                context.update(&bundle.bundle_hash);
                //fill the hash with section & txt indexes
                context.update(&[section_no,txt_no]);
                Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
                    //calc the Hash
                    Entry {data: context.alloc_finalize(&alloc)?},
                    Entry {u64: 0},
                ])?)})
            },
            _ => error(||"Requested value is not providable")
        }
    }
}

pub struct ServerSystem;
impl<'c> SystemContext<'c> for ServerSystem {
    type CE = ServerExternals;
    type RE = ServerExternals;
    type S = SledStore;
    type B = BundleWithHash<'c>;
    type VC = StatefulEntryStoreVerifier<Self::B,ServerSystemDataManager>;
    type EC = StatefulEntryStoreExecutor<Self::B,ServerSystemDataManager>;

    fn parse_bundle<A: ParserAllocator>(data: &[u8], alloc: &'c A) -> Result<Self::B> {
        let txt_bundle:BaseTransactionBundle = Parser::parse_fully(data, CONFIG.max_structural_dept, alloc)?;
        let bundle_hash = HashingDomain::Bundle.hash(&data[..txt_bundle.core.byte_size.unwrap()]);
        Ok(BundleWithHash {
            txt_bundle,
            bundle_hash,
        })
    }
}