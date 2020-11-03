use sanskrit_common::errors::*;
use model::{OpCode, ValueSchema, Entry, Kind};
use sanskrit_common::model::{Hash, SlicePtr, ValueRef, ModuleLink};
use sanskrit_common::arena::{HeapArena, HeapStack, VirtualHeapArena};

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

pub trait ExecutionInterface<'interpreter, 'transaction, 'heap> {
    fn get(&self, idx: usize) -> Result<Entry<'transaction>>;
    fn get_stack(&mut self, tail: bool) -> &mut HeapStack<'interpreter, Entry<'transaction>>;
    fn get_heap(&self) -> &'transaction VirtualHeapArena<'heap>;
    fn process_entry_slice<R: Sized, F: FnOnce(&[u8]) -> R>(kind: Kind, op1: Entry<'transaction>, proc: F) -> R;
}

pub trait RuntimeExternals {
    fn typed_system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool) -> Result<()>;
    fn system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) -> Result<()>;
}

pub trait CompilationExternals {
    fn compile_call<'b,'h>(module:&ModuleLink, fun_idx: u8, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn compile_lit<'b,'h>(module:&ModuleLink, data_idx: u8, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<CompilationResult<'b>>;
    fn get_literal_checker<'b,'h>(module:&ModuleLink, data_idx: u8, len:u16, alloc:&'b HeapArena<'h>) -> Result<ValueSchema<'b>>;
}

pub fn just_gas_and_mem(gas:u64, mem:u64, code:OpCode) -> CompilationResult{
    CompilationResult::OpCodeResult(ExpResources { gas, mem, manifest_stack: 0, frames: 0 }, code)
}