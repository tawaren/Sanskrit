#![no_std]

extern crate sanskrit_compile;
extern crate sanskrit_common;
extern crate sanskrit_chain_code;
extern crate ed25519_consensus;

#[macro_use]
extern crate lazy_static;
extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use sanskrit_common::model::{ValueRef, Hash, ModuleLink};
use sanskrit_chain_code::model::ValueSchema;
use sanskrit_compile::externals::{CompilationResult, CompilationExternals};


pub mod iX;
pub mod uX;
pub mod data;
pub mod ids;
pub mod eddsa;
pub mod _unsafe;
pub mod crypto;

pub trait External:Sync{
    fn compile_lit(&self, data_idx: u8, data:&[u8], caller: &ModuleLink) -> CompilationResult;
    fn get_literal_checker(&self, data_idx: u8, len:u16) -> ValueSchema;
    fn compile_call(&self, fun_idx: u8, params:Vec<ValueRef>, caller:&ModuleLink) -> CompilationResult;
}

//pub static mut EXT_MAP: BTreeMap<Hash, &'static dyn External> = BTreeMap::new();
//pub static mut SYS_HASH: Cell<Hash> = Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);
//pub static mut EDDSA_HASH: Cell<Hash> = Cell::new([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);


/*lazy_static! {
    pub static ref SYS_MODS: [fn(Hash)->();16] = [
            |h|unsafe{EXT_MAP.insert(h, iX::EXT_I8);},        //0
            |h|unsafe{EXT_MAP.insert(h, iX::EXT_I16);},       //1
            |h|unsafe{EXT_MAP.insert(h, iX::EXT_I32);},       //2
            |h|unsafe{EXT_MAP.insert(h, iX::EXT_I64);},       //3
            |h|unsafe{EXT_MAP.insert(h, iX::EXT_I128);},      //4
            |h|unsafe{EXT_MAP.insert(h, uX::EXT_U8);},        //5
            |h|unsafe{EXT_MAP.insert(h, uX::EXT_U16);},       //6
            |h|unsafe{EXT_MAP.insert(h, uX::EXT_U32);},       //7
            |h|unsafe{EXT_MAP.insert(h, uX::EXT_U64);},       //8
            |h|unsafe{EXT_MAP.insert(h, uX::EXT_U128);},      //9
            |h|unsafe{EXT_MAP.insert(h, data::EXT_DATA);},    //10
            |h|unsafe{EXT_MAP.insert(h, ids::EXT_IDS);},      //11
            |h|unsafe{SYS_HASH.set(h);},                      //12
            |h|unsafe{EXT_MAP.insert(h, eddsa::EXT_ECDSA);},  //13
            |h|unsafe{EXT_MAP.insert(h,_unsafe::EXT_UNSAFE);},//14
            |h|unsafe{EDDSA_HASH.set(h);},                    //15
    ];
}*/



/*
pub struct ServerExternals;
impl CompilationExternals for ServerExternals {
    fn compile_call(module: &ModuleLink, fun_idx: u8, params: Vec<ValueRef>, caller: &ModuleLink) -> CompilationResult {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller)
        }
    }

    fn compile_lit(module: &ModuleLink, data_idx: u8, data: &[u8], caller: &ModuleLink) -> CompilationResult {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller)
        }
    }

    fn get_literal_checker(module: &ModuleLink, data_idx: u8, len: u16) -> ValueSchema {
        match EXT_MAP.lock().get(module.module_hash()) {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
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
}*/

pub trait StaticExternalsProvider {
    fn get(hash:&Hash) -> Option<&'static dyn External>;
}

pub struct StaticExternals<P:StaticExternalsProvider>(PhantomData<P>);
impl<P:StaticExternalsProvider> CompilationExternals for StaticExternals<P>  {
    fn compile_call(module: &ModuleLink, fun_idx: u8, params: Vec<ValueRef>, caller: &ModuleLink) -> CompilationResult {
        match P::get(module.module_hash()) {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
            Some(ref imp) => imp.compile_call(fun_idx, params, caller)
        }
    }

    fn compile_lit(module: &ModuleLink, data_idx: u8, data: &[u8], caller: &ModuleLink) -> CompilationResult {
        match P::get(module.module_hash()) {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
            Some(ref imp) => imp.compile_lit(data_idx, data, caller)
        }
    }

    fn get_literal_checker(module: &ModuleLink, data_idx: u8, len: u16) -> ValueSchema {
        match P::get(module.module_hash())  {
            None => panic!("Implementation for external module is missing (Hash: {:?})", module.module_hash()),
            Some(ref imp) => imp.get_literal_checker(data_idx, len)
        }
    }
}

