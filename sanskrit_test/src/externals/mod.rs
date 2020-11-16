use sanskrit_common::errors::*;
use sanskrit_common::model::{ModuleLink, SlicePtr, ValueRef, Hash, Ptr};
use sanskrit_common::arena::{HeapArena, VirtualHeapArena};
use sanskrit_interpreter::model::{ValueSchema, Kind, Entry, Adt, RuntimeType};
use std::collections::BTreeMap;
use sanskrit_common::encoding::*;
use sanskrit_runtime::system::SystemContext;
use sanskrit_common::hashing::HashingDomain;
use sanskrit_runtime::model::{BundleWithHash, BaseTransactionBundle};
use sanskrit_runtime::direct_stored::{StatefulEntryStoreVerifier, StatefulEntryStoreExecutor, SystemDataManager};
use sanskrit_runtime::CONFIG;
use sanskrit_memory_store::BTreeMapStore;
use sanskrit_compile::externals::{CompilationExternals, CompilationResult};
use sanskrit_interpreter::externals::{RuntimeExternals, ExecutionInterface};
use externals::crypto::{plain_hash, join_hash, ecdsa_verify};

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
    static ref EXT_MAP: BTreeMap<Hash, &'static dyn External> = {
        let mut map = BTreeMap::new();
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/inti8.hash"),5, &NoCustomAlloc()).unwrap(), i8::EXT_I8);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/inti16.hash"),5, &NoCustomAlloc()).unwrap(), i16::EXT_I16);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/inti32.hash"),5, &NoCustomAlloc()).unwrap(), i32::EXT_I32);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/inti64.hash"),5, &NoCustomAlloc()).unwrap(), i64::EXT_I64);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/inti128.hash"),5, &NoCustomAlloc()).unwrap(), i128::EXT_I128);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/intu8.hash"),5, &NoCustomAlloc()).unwrap(), u8::EXT_U8);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/intu16.hash"),5, &NoCustomAlloc()).unwrap(), u16::EXT_U16);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/intu32.hash"),5, &NoCustomAlloc()).unwrap(), u32::EXT_U32);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/intu64.hash"),5, &NoCustomAlloc()).unwrap(), u64::EXT_U64);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/intu128.hash"),5, &NoCustomAlloc()).unwrap(), u128::EXT_U128);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/data.hash"),5, &NoCustomAlloc()).unwrap(), data::EXT_DATA);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/ids.hash"),5, &NoCustomAlloc()).unwrap(), ids::EXT_IDS);
        map.insert(Parser::parse_fully(include_bytes!("../../scripts/out/ecdsa.hash"),5, &NoCustomAlloc()).unwrap(), eddsa::EXT_ECDSA);
        //map.insert(Parser::parse_fully(include_bytes!("../scripts/out/unsafe.hash"),5, &NoCustomAlloc()).unwrap(), _unsafe::EXT_UNSAFE);
        map
     };
}

lazy_static! {
  static ref SYS_HASH: Hash = Parser::parse_fully(include_bytes!("../../scripts/out/system.hash"),5, &NoCustomAlloc()).unwrap();
}

pub struct ScriptExternals;
impl CompilationExternals for ScriptExternals {
    fn compile_call<'b, 'h>(module: &ModuleLink, fun_idx: u8, params: SlicePtr<'b, ValueRef>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller, alloc)
        }
    }

    fn compile_lit<'b, 'h>(module: &ModuleLink, data_idx: u8, data: SlicePtr<'b, u8>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller, alloc)
        }
    }

    fn get_literal_checker<'b, 'h>(module: &ModuleLink, data_idx: u8, len: u16, alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match EXT_MAP.get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.get_literal_checker(data_idx, len, alloc)
        }
    }
}

impl RuntimeExternals for ScriptExternals {
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

pub struct ScriptSystemDataManager;
impl<'c> SystemDataManager<BundleWithHash<'c>> for ScriptSystemDataManager {

    fn providable_size(typ: Ptr<RuntimeType>) -> Result<u32> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == *SYS_HASH && offset == 1 => {
                Ok((2*Hash::SIZE + 4*Entry::SIZE) as u32)
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }

    fn providable_gas(typ: Ptr<RuntimeType>) -> Result<u64> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == *SYS_HASH && offset == 1 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (4 as u64);
                Ok(2*hash_alloc + pack)
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }


    fn is_chain_value(typ: Ptr<RuntimeType>) -> bool {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == *SYS_HASH && offset == 0 => true,
            _ => false
        }
    }

    //This means we can only provide 1 value per Txt
    fn provided_value_key(_typ: Ptr<RuntimeType>, section_no:u8,  txt_no:u8) -> Option<Vec<u8>> {
        Some(vec![section_no,txt_no])
    }

    fn create_provided_value<'a, 'h>(bundle: &BundleWithHash, _typ: Ptr<RuntimeType>, alloc: &'a VirtualHeapArena<'h>, block_no: u64, section_no:u8,  txt_no:u8) -> Result<Entry<'a>> {
        let mut context = HashingDomain::Derive.get_domain_hasher();
        //fill the hash with first value
        context.update(&bundle.bundle_hash);
        //fill the hash with second value
        context.update(&[section_no,txt_no]);
        //calc the Hash
        let txt_id = context.alloc_finalize(&alloc)?;
        Ok(Entry{adt: Adt(0,alloc.copy_alloc_slice(&[
            Entry {data: txt_id},
            Entry {data: alloc.copy_alloc_slice(&bundle.bundle_hash)?},
            Entry {u64: block_no},
            Entry {u8: section_no},
            Entry {u8: txt_no},
            Entry {u64: 0}
        ])?)})
    }
}

pub struct ScriptSystem;
impl<'c> SystemContext<'c> for ScriptSystem {
    type CE = ScriptExternals;
    type RE = ScriptExternals;
    type S = BTreeMapStore;
    type B = BundleWithHash<'c>;
    type VC = StatefulEntryStoreVerifier<Self::B,ScriptSystemDataManager>;
    type EC = StatefulEntryStoreExecutor<Self::B,ScriptSystemDataManager>;

    fn parse_bundle<A: ParserAllocator>(data: &[u8], alloc: &'c A) -> Result<Self::B> {
        let txt_bundle:BaseTransactionBundle = Parser::parse_fully(data, CONFIG.max_structural_dept, alloc)?;
        let bundle_hash = HashingDomain::Bundle.hash(&data[..txt_bundle.core.byte_size.unwrap()]);
        Ok(BundleWithHash {
            txt_bundle,
            bundle_hash,
        })
    }

}