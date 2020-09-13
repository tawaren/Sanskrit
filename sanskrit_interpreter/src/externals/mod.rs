use sanskrit_common::errors::*;
use model::{OpCode, ValueSchema};
use sanskrit_common::model::{Hash, SlicePtr, ValueRef, ModuleLink};
use sanskrit_common::arena::{HeapArena};
use sanskrit_common::encoding::*;


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

pub trait External:Sync{
    fn compile_lit<'b,'h>(&self, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(&self, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
    fn compile_call<'b,'h>(&self, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
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