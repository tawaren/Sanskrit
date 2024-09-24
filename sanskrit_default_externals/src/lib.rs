extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_interpreter;
extern crate ed25519_dalek;

#[macro_use]
extern crate lazy_static;

use sanskrit_common::errors::*;
use sanskrit_common::model::{ValueRef, Hash, ModuleLink};
use sanskrit_common::model::{SlicePtr};
use std::sync::Mutex;
use std::collections::BTreeMap;
use std::cell::Cell;
use crypto::{ecdsa_verify, join_hash, plain_hash};
use sanskrit_common::arena::HeapArena;
use sanskrit_interpreter::model::{Kind, ValueSchema};
use sanskrit_common::encoding::{VirtualSize, ParserAllocator};
use sanskrit_common::hashing::HashingDomain;
use sanskrit_interpreter::externals::{RuntimeExternals, ExecutionInterface};
use sanskrit_compile::externals::{CompilationResult, CompilationExternals};


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

lazy_static! {
    pub static ref SYS_MODS: [fn(Hash)->();16] = [
            |h|{EXT_MAP.lock().unwrap().insert(h, i8::EXT_I8);},        //0
            |h|{EXT_MAP.lock().unwrap().insert(h, i16::EXT_I16);},      //1
            |h|{EXT_MAP.lock().unwrap().insert(h, i32::EXT_I32);},      //2
            |h|{EXT_MAP.lock().unwrap().insert(h, i64::EXT_I64);},      //3
            |h|{EXT_MAP.lock().unwrap().insert(h, i128::EXT_I128);},    //4
            |h|{EXT_MAP.lock().unwrap().insert(h, u8::EXT_U8);},        //5
            |h|{EXT_MAP.lock().unwrap().insert(h, u16::EXT_U16);},      //6
            |h|{EXT_MAP.lock().unwrap().insert(h, u32::EXT_U32);},      //7
            |h|{EXT_MAP.lock().unwrap().insert(h, u64::EXT_U64);},      //8
            |h|{EXT_MAP.lock().unwrap().insert(h, u128::EXT_U128);},    //9
            |h|{EXT_MAP.lock().unwrap().insert(h, data::EXT_DATA);},    //10
            |h|{EXT_MAP.lock().unwrap().insert(h, ids::EXT_IDS);},      //11
            |h|{SYS_HASH.lock().unwrap().set(h);},                      //12
            |h|{EXT_MAP.lock().unwrap().insert(h, eddsa::EXT_ECDSA);},  //13
            |h|{EXT_MAP.lock().unwrap().insert(h,_unsafe::EXT_UNSAFE);},//14
            |h|{EDDSA_HASH.lock().unwrap().set(h);},                    //15
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

impl RuntimeExternals for ServerExternals {

    fn typed_system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Hash
            0 => plain_hash(interface, kind, values[0], tail),
            _ => unreachable!("Non Existent typed System Call")
        }
    }

    fn system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Derive
            0 => join_hash(interface, values[0], values[1], HashingDomain::Derive, tail),
            //EcDsaVerify
            1 => ecdsa_verify(interface, values[0], values[1], values[2], tail),
            _ => unreachable!("Non Existent System Call")
        }
    }
}
