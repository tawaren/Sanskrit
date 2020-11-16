use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use externals::External;
use sanskrit_compile::externals::{just_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind};

pub const EXT_ECDSA:&'static dyn External = &Ecdsa;

pub struct Ecdsa;
impl External for Ecdsa{
    /*
    public extType(32) <Copy,Drop,Persist,Primitive,Value,Unbound> Pk;
    public extType(64) <Copy,Drop,Persist,Primitive,Value,Unbound> Sig;
    */
    fn compile_lit<'b, 'h>(&self, data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match data_idx {
            0 => Ok(just_gas_and_mem(14, 32,OpCode::Data(data))),
            _ => Ok(just_gas_and_mem(15, 64,OpCode::Data(data))),
        }
    }

    fn get_literal_checker<'b, 'h>(&self, data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match data_idx {
            0 => Ok(ValueSchema::Data(32)),
            _ => Ok(ValueSchema::Data(64)),
        }
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            //public extFun derivePublicId(pk:.Pk):(pub:.PublicId);
            0 => Ok(just_gas_and_mem(65, Hash::SIZE as u64, OpCode::TypedSysInvoke(0, Kind::Data, params))),
            /*public extFun verify1(msg:Data.Data1, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify2(msg:Data.Data2, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify4(msg:Data.Data4, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify8(msg:Data.Data8, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify12(msg:Data.Data12, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify16(msg:Data.Data16, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify20(msg:Data.Data20, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify24(msg:Data.Data24, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify28(msg:Data.Data28, pk:.Pk, sig:.Sig):(res:Bool.Bool);
              public extFun verify32(msg:Data.Data32, pk:.Pk, sig:.Sig):(res:Bool.Bool);
            */
            //Todo: measure this it is guessed based on ethereum gas costs for similar operations
            _ => Ok(just_gas_and_mem(4500, 0, OpCode::SysInvoke(1, params))),

        }
    }
}