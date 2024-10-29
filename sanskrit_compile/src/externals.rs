use alloc::vec::Vec;
use sanskrit_chain_code::model::{OpCode, ValueSchema};
use sanskrit_common::model::{ModuleLink, ValueRef};

pub enum CompilationResult {
    OpCodeResult(OpCode),
    ReorderResult(Vec<u8>)
}

pub trait CompilationExternals {
    fn compile_call(module:&ModuleLink, fun_idx: u8, params:Vec<ValueRef>, caller:&ModuleLink) -> CompilationResult;
    fn compile_lit(module:&ModuleLink, data_idx: u8, data:&[u8], caller: &ModuleLink) -> CompilationResult;
    fn get_literal_checker(module:&ModuleLink, data_idx: u8, len:u16) -> ValueSchema;
}