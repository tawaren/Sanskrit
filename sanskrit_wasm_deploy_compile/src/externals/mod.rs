use std::sync::Mutex;
use sanskrit_common::errors::*;
use sanskrit_common::model::{ModuleLink, SlicePtr, ValueRef, Hash};
use sanskrit_common::arena::HeapArena;
use std::collections::BTreeMap;
use std::cell::Cell;
use sanskrit_compile::externals::{CompilationExternals, CompilationResult};
use sanskrit_interpreter::model::ValueSchema;

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
            |_|{},                                                       //15
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