#![no_std]

extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_chain_code;
extern crate ed25519_consensus;

#[macro_use]
extern crate lazy_static;
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::Cell;
use spin::Mutex;

use sanskrit_common::model::{ValueRef, Hash, ModuleLink};
use crypto::{ecdsa_verify, join_hash, plain_hash};
use sanskrit_chain_code::model::{Kind, ValueSchema};
use sanskrit_common::hashing::HashingDomain;
use sanskrit_chain_code::{RuntimeExternals, ExecutionInterface};
use sanskrit_compile::externals::{CompilationResult, CompilationExternals};


pub mod iX;
pub mod uX;
pub mod data;
pub mod ids;
pub mod eddsa;
pub mod _unsafe;
pub mod crypto;

pub trait External:Sync{
    fn compile_lit(&self, data_idx: u8, data:&[u8], caller: &Hash) -> Result<CompilationResult>;
    fn get_literal_checker(&self, data_idx: u8, len:u16) -> Result<ValueSchema>;
    fn compile_call(&self, fun_idx: u8, params:Vec<ValueRef>, caller:&Hash) -> Result<CompilationResult>;
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
            |h|{EXT_MAP.lock().insert(h, iX::EXT_I8);},        //0
            |h|{EXT_MAP.lock().insert(h, iX::EXT_I16);},      //1
            |h|{EXT_MAP.lock().insert(h, iX::EXT_I32);},      //2
            |h|{EXT_MAP.lock().insert(h, iX::EXT_I64);},      //3
            |h|{EXT_MAP.lock().insert(h, iX::EXT_I128);},    //4
            |h|{EXT_MAP.lock().insert(h, uX::EXT_U8);},        //5
            |h|{EXT_MAP.lock().insert(h, uX::EXT_U16);},      //6
            |h|{EXT_MAP.lock().insert(h, uX::EXT_U32);},      //7
            |h|{EXT_MAP.lock().insert(h, uX::EXT_U64);},      //8
            |h|{EXT_MAP.lock().insert(h, uX::EXT_U128);},    //9
            |h|{EXT_MAP.lock().insert(h, data::EXT_DATA);},    //10
            |h|{EXT_MAP.lock().insert(h, ids::EXT_IDS);},      //11
            |h|{SYS_HASH.lock().set(h);},                      //12
            |h|{EXT_MAP.lock().insert(h, eddsa::EXT_ECDSA);},  //13
            |h|{EXT_MAP.lock().insert(h,_unsafe::EXT_UNSAFE);},//14
            |h|{EDDSA_HASH.lock().set(h);},                    //15
    ];
}

pub struct ServerExternals;
impl CompilationExternals for ServerExternals {
    fn compile_call(module: &ModuleLink, fun_idx: u8, params: Vec<ValueRef>, caller: &Hash) -> CompilationResult {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing"),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller)
        }
    }

    fn compile_lit(module: &ModuleLink, data_idx: u8, data: &[u8], caller: &Hash) -> CompilationResult {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing"),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller)
        }
    }

    fn get_literal_checker(module: &ModuleLink, data_idx: u8, len: u16) -> ValueSchema {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing"),
            Some(ref imp) => imp.get_literal_checker(data_idx, len)
        }
    }
}

impl RuntimeExternals for ServerExternals {

    fn typed_system_call<I:ExecutionInterface>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool){
        match id {
            //Hash
            0 => plain_hash(interface, kind, values[0], tail),
            _ => unreachable!("Non Existent typed System Call")
        }
    }

    fn system_call<I:ExecutionInterface>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) {
        match id {
            //Derive
            0 => join_hash(interface, values[0], values[1], HashingDomain::Derive, tail),
            //EcDsaVerify
            1 => ecdsa_verify(interface, values[0], values[1], values[2], tail),
            _ => unreachable!("Non Existent System Call")
        }
    }
}
