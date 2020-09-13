use sanskrit_common::errors::*;
use sanskrit_interpreter::externals::{CompilationExternals, CompilationResult};
use sanskrit_common::model::{ModuleLink, SlicePtr, ValueRef, Hash};
use sanskrit_common::arena::HeapArena;
use sanskrit_interpreter::model::ValueSchema;
use std::collections::BTreeMap;
use sanskrit_common::encoding::*;
use sanskrit_interpreter::externals::External;
use sanskrit_interpreter::*;
use sanskrit_runtime::system::System;

lazy_static! {
    static ref EXT_MAP: BTreeMap<Hash, &'static dyn External> = {
        let mut map = BTreeMap::new();
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/inti8.hash"),5, &NoCustomAlloc()).unwrap(), externals::i8::EXT_I8);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/inti16.hash"),5, &NoCustomAlloc()).unwrap(), externals::i16::EXT_I16);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/inti32.hash"),5, &NoCustomAlloc()).unwrap(), externals::i32::EXT_I32);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/inti64.hash"),5, &NoCustomAlloc()).unwrap(), externals::i64::EXT_I64);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/inti128.hash"),5, &NoCustomAlloc()).unwrap(), externals::i128::EXT_I128);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/intu8.hash"),5, &NoCustomAlloc()).unwrap(), externals::u8::EXT_U8);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/intu16.hash"),5, &NoCustomAlloc()).unwrap(), externals::u16::EXT_U16);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/intu32.hash"),5, &NoCustomAlloc()).unwrap(), externals::u32::EXT_U32);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/intu64.hash"),5, &NoCustomAlloc()).unwrap(), externals::u64::EXT_U64);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/intu128.hash"),5, &NoCustomAlloc()).unwrap(), externals::u128::EXT_U128);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/data.hash"),5, &NoCustomAlloc()).unwrap(), externals::data::EXT_DATA);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/ids.hash"),5, &NoCustomAlloc()).unwrap(), externals::ids::EXT_IDS);
        map.insert(Parser::parse_fully(include_bytes!("../scripts/out/ecdsa.hash"),5, &NoCustomAlloc()).unwrap(), externals::eddsa::EXT_ECDSA);
        map
     };
}

lazy_static! {
  static ref SYS_HASH: Hash = Parser::parse_fully(include_bytes!("../scripts/out/system.hash"),5, &NoCustomAlloc()).unwrap();
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
pub struct ScriptSystem;
impl System for ScriptSystem {
    fn system_module(&self) -> Hash {
        *SYS_HASH
    }

    fn entry_offset(&self) -> u8 {
        0
    }

    fn context_offset(&self) -> u8 {
        1
    }

    fn txt_hash_offset(&self) -> u8 {
        0
    }

    fn code_hash_offset(&self) -> u8 {
        1
    }

    fn full_hash_offset(&self) -> u8 {
        2
    }

    fn unique_id_offset(&self) -> u8 {
        3
    }
}