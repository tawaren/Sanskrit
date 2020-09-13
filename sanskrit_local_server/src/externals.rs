use std::sync::Mutex;
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
use std::cell::Cell;

lazy_static! {
    pub static ref EXT_MAP: Mutex<BTreeMap<Hash, &'static dyn External>> = Mutex::new(BTreeMap::new());
}

lazy_static! {
    pub static ref SYS_HASH: Mutex<Cell<Hash>> = Mutex::new(Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]));
}


lazy_static! {
    pub static ref SYS_MODS: [fn(Hash)->();14] = [
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i8::EXT_I8);},      //0
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i16::EXT_I16);},    //1
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i32::EXT_I32);},    //2
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i64::EXT_I64);},    //3
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::i128::EXT_I128);},  //4
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u8::EXT_U8);},      //5
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u16::EXT_U16);},    //6
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u32::EXT_U32);},    //7
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u64::EXT_U64);},    //8
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::u128::EXT_U128);},  //9
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::data::EXT_DATA);},  //10
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::ids::EXT_IDS);},    //11
            |h|{SYS_HASH.lock().unwrap().set(h);},                              //12
            |h|{EXT_MAP.lock().unwrap().insert(h,externals::eddsa::EXT_ECDSA);},//13
    ];
}



pub struct ServerExternals;
impl CompilationExternals for ServerExternals {
    fn compile_call<'b, 'h>(module: &ModuleLink, fun_idx: u8, params: SlicePtr<'b, ValueRef>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller, alloc)
        }
    }

    fn compile_lit<'b, 'h>(module: &ModuleLink, data_idx: u8, data: SlicePtr<'b, u8>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller, alloc)
        }
    }

    fn get_literal_checker<'b, 'h>(module: &ModuleLink, data_idx: u8, len: u16, alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match EXT_MAP.lock().unwrap().get(&module.to_hash()) {
            None => error(||"Implementation for external module is missing"),
            Some(ref imp) => imp.get_literal_checker(data_idx, len, alloc)
        }
    }
}

pub struct ServerSystem;

impl System for ServerSystem {
    fn system_module(&self) -> Hash {
        SYS_HASH.lock().unwrap().get()
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