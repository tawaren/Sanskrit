use sanskrit_interpreter::model::{OpCode, ValueSchema};
use sanskrit_common::model::{SlicePtr, ModuleLink, ValueRef, Hash};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ExpResources {
    pub gas:u64,
    pub mem:u64,
    pub manifest_stack: u32,
    pub frames: u32,
}

impl ExpResources {
    pub fn empty() -> Self {
        ExpResources {
            gas: 0,
            mem: 0,
            manifest_stack: 0,
            frames: 0
        }
    }
}

pub enum CompilationResult<'b> {
    OpCodeResult(ExpResources, OpCode<'b>),
    ReorderResult(SlicePtr<'b,u8>)
}

pub trait CompilationExternals {
    fn compile_call<'b,'h>(module:&ModuleLink, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn compile_lit<'b,'h>(module:&ModuleLink, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(module:&ModuleLink, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
}

pub fn just_gas_and_mem(gas:u64, mem:u64, code:OpCode) -> CompilationResult{
    CompilationResult::OpCodeResult(ExpResources { gas, mem, manifest_stack: 0, frames: 0 }, code)
}