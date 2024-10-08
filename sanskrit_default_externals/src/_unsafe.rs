use alloc::vec::Vec;
use sanskrit_common::model::{Hash, ValueRef};
use sanskrit_common::errors::*;
use sanskrit_compile::externals::CompilationResult;
use sanskrit_chain_code::model::ValueSchema;
use crate::External;

pub const EXT_UNSAFE:&'static dyn External = &Unsafe;

pub struct Unsafe;
impl External for Unsafe{
    //local external(0) standard temporary data Unsafe[phantom T]
    fn compile_lit(&self, _data_idx: u8, _data:&[u8], _caller: &Hash) -> Result<CompilationResult> {
        error(||"Unsafe can not be used in literal creation")
    }

    fn get_literal_checker(&self, _data_idx: u8, _len: u16) -> Result<ValueSchema> {
        error(||"Unsafe can not be used in transaction parameters")
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, _params: Vec<ValueRef>, _caller: &Hash) -> Result<CompilationResult> {
        Ok(match fun_idx {
            //local external function _unProject[T](t:project(T)):(res:T)
            //local external function _packUnsafe[T](t:T):(res:Unsafe[T])
            //local external function _unpackUnsafe[T](t:Unsafe[T]):(res:T)
            //local external function _copy[T](t:T):(res:T)
            x if x <= 3 => CompilationResult::ReorderResult((&[0]).to_vec()),
            //local external function _consume[T](consume t:T):()
            4 => CompilationResult::ReorderResult(Vec::with_capacity(0)),
            _ => return error(||"External call is not defined")
        })
    }
}
