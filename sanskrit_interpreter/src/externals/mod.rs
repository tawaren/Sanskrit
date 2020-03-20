use alloc::collections::BTreeMap;
use sanskrit_common::errors::*;
use model::{OpCode, ValueSchema};
use sanskrit_common::model::{Hash, SlicePtr, ValueRef, ModuleLink};
use sanskrit_common::arena::{HeapArena};


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


pub trait External:Sync{
    fn compile_lit<'b,'h>(&self, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(&self, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
    fn compile_call<'b,'h>(&self, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
}

pub fn check_all_hashes() {
    i8::check_hash();
    i16::check_hash();
    i32::check_hash();
    i64::check_hash();
    i128::check_hash();
    u8::check_hash();
    u16::check_hash();
    u32::check_hash();
    u64::check_hash();
    u128::check_hash();
    data::check_hash();
    ids::check_hash();
}

lazy_static! {
    static ref EXT_MAP: BTreeMap<Hash, &'static dyn External> = {
        check_all_hashes();
        let mut map = BTreeMap::new();
        map.insert(i8::MODULE, i8::EXT_I8);
        map.insert(i16::MODULE, i16::EXT_I16);
        map.insert(i32::MODULE, i32::EXT_I32);
        map.insert(i64::MODULE, i64::EXT_I64);
        map.insert(i128::MODULE, i128::EXT_I128);
        map.insert(u8::MODULE, u8::EXT_U8);
        map.insert(u16::MODULE, u16::EXT_U16);
        map.insert(u32::MODULE, u32::EXT_U32);
        map.insert(u64::MODULE, u64::EXT_U64);
        map.insert(u128::MODULE, u128::EXT_U128);
        map.insert(data::MODULE, data::EXT_DATA);
        map.insert(ids::MODULE, ids::EXT_IDS);
        map
     };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CallResources {
    pub max_gas:u64,
    pub max_mem:u64,
    pub max_manifest_stack: u32,
    pub max_frames: u32,
}

pub enum CompilationResult<'b> {
    OpCodeResult(CallResources, OpCode<'b>),
    ReorderResult(SlicePtr<'b,u8>)

}

pub trait CompilationExternals {
    fn compile_call<'b,'h>(module:&ModuleLink, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn compile_lit<'b,'h>(module:&ModuleLink, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(module:&ModuleLink, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
}

pub fn just_gas_and_mem(gas:u64, mem:u64, code:OpCode) -> CompilationResult{
    CompilationResult::OpCodeResult(CallResources{ max_gas: gas, max_mem: mem, max_manifest_stack: 0, max_frames: 0 }, code)
}

pub struct Externals;
impl CompilationExternals for Externals {
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


    /*
    fn compile_call<'b,'h>(fun_idx: u16, params:SlicePtr<'b,ValueRef>, caller: &Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        //toI / toU will be 11
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            0 => return Ok(CompilationResult::ReorderResult(alloc.copy_alloc_slice(&[0])?)),
            1 => just_gas_and_mem(70, 20, OpCode::Derive(params[0], params[1])),
            2 => just_gas_and_mem(13, 20, OpCode::Data(alloc.copy_alloc_slice(caller)?)),
            //currently we have max 32 Byte:
            x if x >= 4 && x < 16 => just_gas_and_mem(14, 0,OpCode::Eq(get_kind(x-4),params[0], params[1])),
            x if x >= 16 && x < 28 => just_gas_and_mem(13, 0,OpCode::Lt(get_kind(x-16),params[0], params[1])),
            x if x >= 28 && x < 40 => just_gas_and_mem(13, 0,OpCode::Lte(get_kind(x-28),params[0], params[1])),
            x if x >= 40 && x < 52 => just_gas_and_mem(13, 0,OpCode::Gt(get_kind(x-40),params[0], params[1])),
            x if x >= 52 && x < 64 => just_gas_and_mem(13, 0,OpCode::Gte(get_kind(x-52),params[0], params[1])),
            x if x >= 64 && x < 76 => just_gas_and_mem(12, 0,OpCode::Add(get_kind(x-64),params[0], params[1])),
            x if x >= 76 && x < 88 => just_gas_and_mem(12, 0,OpCode::Sub(get_kind(x-76),params[0], params[1])),
            x if x >= 88 && x < 100 => just_gas_and_mem(17, 0,OpCode::Div(get_kind(x-88),params[0], params[1])),
            x if x >= 100 && x < 112 => just_gas_and_mem(13, 0,OpCode::Mul(get_kind(x-100),params[0], params[1])),
            x if x >= 112 && x < 124 => just_gas_and_mem(18, get_size(x-112), OpCode::ToData(get_kind(x-112),params[0])),
            x if x >= 124 && x < 136 => just_gas_and_mem(65, 20, OpCode::Hash(get_kind(x-124), params[0])),
            n => return error(||"External call is not defined")
        })
    }

    fn compile_lit<'b,'h>(lit_idx: u16, data:SlicePtr<'b,u8>, _caller: &Hash,  _alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match lit_idx {
            0 => match data.len() {
                1 => just_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U8)),
                2 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::U16)),
                4 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::U32)),
                8 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::U64)),
                16 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::U128)),
                s => return error(||"External lit is not defined")
            }
            1 => match data.len() {
                1 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::I8)),
                2 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::I16)),
                4 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::I32)),
                8 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::I64)),
                16 => just_gas_and_mem(7, 0,OpCode::SpecialLit(data, LitDesc::I128)),
                s => return error(||"External lit is not defined")
            }
            2 => just_gas_and_mem((13 + data.len()/50) as u64, data.len() as u64,OpCode::Data(data)),
            n => return error(||"External lit is not defined")
        })
    }

    fn get_literal_checker<'b, 'h>(lit_idx: u16, size: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(match lit_idx {
            0 => ValueSchema::Unsigned(size as u8),
            1 => ValueSchema::Signed(size as u8),
            n => return error(||"External lit is not defined")
        })
    }*/
}