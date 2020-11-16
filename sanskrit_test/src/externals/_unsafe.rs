use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_compile::externals::CompilationResult;
use sanskrit_interpreter::model::ValueSchema;
use externals::External;

pub const EXT_UNSAFE:&'static dyn External = &Unsafe;

pub struct Unsafe;
impl External for Unsafe{
    //private extType(0) <Copy,Drop,Value,Unbound> Unsafe[phantom T];
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, _data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        error(||"Unsafe can not be used in literal creation")
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        error(||"Unsafe can not be used in transaction parameters")
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, _params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //private extFun _unproject[<Unbound> T](t:%T):(res:T);
            //private extFun _packUnsafe[<Unbound> T](t:T):(res:.Unsafe[T]);
            //private extFun _unpackUnsafe[<Unbound> T](t:.Unsafe[T]):(res:T);
            //private extFun _copy[<Unbound> T](t:T):(res:T);
            x if x <= 3 => CompilationResult::ReorderResult(alloc.copy_alloc_slice(&[0])?),
            //private extFun _consume[T](consume t:T):();
            4 => CompilationResult::ReorderResult(SlicePtr::empty()),
            _ => return error(||"External call is not defined")
        })
    }
}