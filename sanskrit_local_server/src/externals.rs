use std::sync::Mutex;
use sanskrit_common::errors::*;
use sanskrit_interpreter::externals::{CompilationExternals, CompilationResult, RuntimeExternals, ExecutionInterface};
use sanskrit_common::model::{ModuleLink, SlicePtr, ValueRef, Hash, Ptr};
use sanskrit_common::arena::{HeapArena, VirtualHeapArena};
use sanskrit_interpreter::model::{ValueSchema, Kind, Entry, Adt, RuntimeType};
use std::collections::BTreeMap;
use sanskrit_interpreter::externals::External;
use sanskrit_interpreter::externals::crypto::*;
use sanskrit_interpreter::*;
use sanskrit_runtime::system::SystemContext;
use std::cell::Cell;
use sanskrit_common::hashing::HashingDomain;
use sanskrit_sled_store::SledStore;
use sanskrit_runtime::direct_stored::{StatefulEntryStoreVerifier, SystemDataManager, StatefulEntryStoreExecutor};
use sanskrit_common::encoding::{VirtualSize, ParserAllocator, Parser};
use sanskrit_runtime::model::{BundleWithHash, BaseTransactionBundle};
use sanskrit_runtime::CONFIG;

lazy_static! {
    pub static ref EXT_MAP: Mutex<BTreeMap<Hash, &'static dyn External>> = Mutex::new(BTreeMap::new());
}

lazy_static! {
    pub static ref SYS_HASH: Mutex<Cell<Hash>> = Mutex::new(Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
}


lazy_static! {
    pub static ref SYS_MODS: [fn(Hash)->();15] = [
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i8::EXT_I8);},          //0
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i16::EXT_I16);},        //1
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i32::EXT_I32);},        //2
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i64::EXT_I64);},        //3
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i128::EXT_I128);},      //4
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u8::EXT_U8);},          //5
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u16::EXT_U16);},        //6
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u32::EXT_U32);},        //7
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u64::EXT_U64);},        //8
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u128::EXT_U128);},      //9
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::data::EXT_DATA);},      //10
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::ids::EXT_IDS);},        //11
            |h|{SYS_HASH.lock().unwrap().set(h);},                                  //12
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::eddsa::EXT_ECDSA);},    //13
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::_unsafe::EXT_UNSAFE);}, //14

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
                Ok((2*Hash::SIZE + 6*Entry::SIZE) as u32)
            }
            _ => return error(||"Provided value parameter must be of a supported type")
        }
    }

    fn providable_gas(typ: Ptr<RuntimeType>) -> Result<u64> {
        match *typ {
            RuntimeType::Custom { module, offset, .. } if module == SYS_HASH.lock().unwrap().get() && offset == 1 => {
                let hash_alloc = (13 + 20/50) as u64;
                let pack = 13 + (6 as u64);
                let hash_cost = 65;
                Ok(2*hash_alloc + pack + hash_cost)
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