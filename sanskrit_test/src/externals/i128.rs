use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};
use externals::External;

pub const EXT_I128:&'static dyn External = &I128;

pub struct I128;
impl External for I128{
    //public(create) extType(16) <Copy,Drop,Persist,Value,Unbound> I128;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::I128)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Signed(16))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //public extFun eq(num1:.I128, num2:.I128):(res:Bool.Bool);
            0 => just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::I128, params[0], params[1])),
            //public extFun lt(num1:.I128, num2:.I128):(res:Bool.Bool);
            1 => just_local_gas_and_mem(13, 0, OpCode::Lt(Kind::I128, params[0], params[1])),
            //public extFun lte(num1:.I128, num2:.I128):(res:Bool.Bool);
            2 => just_local_gas_and_mem(13, 0, OpCode::Lte(Kind::I128, params[0], params[1])),
            //public extFun gt(num1:.I128, num2:.I128):(res:Bool.Bool);
            3 => just_local_gas_and_mem(13, 0, OpCode::Gt(Kind::I128, params[0], params[1])),
            //public extFun gte(num1:.I128, num2:.I128):(res:Bool.Bool);
            4 => just_local_gas_and_mem(13, 0, OpCode::Gte(Kind::I128, params[0], params[1])),
            //public extFun add(num1:.I128, num2:.I128):(res:.I128);
            5 => just_local_gas_and_mem(12, 0, OpCode::Add(Kind::I128, params[0], params[1])),
            //public extFun sub(num1:.I128, num2:.I128):(res:.I128);
            6 => just_local_gas_and_mem(12, 0, OpCode::Sub(Kind::I128, params[0], params[1])),
            //public extFun div(num1:.I128, num2:.I128):(res:.I128);
            7 => just_local_gas_and_mem(17, 0, OpCode::Div(Kind::I128, params[0], params[1])),
            //public extFun mul(num1:.I128, num2:.I128):(res:.I128);
            8 => just_local_gas_and_mem(13, 0, OpCode::Mul(Kind::I128, params[0], params[1])),
            //public transactional extFun and(num1:.I128, num2:.I128):(res:.I128);
            9 => just_local_gas_and_mem(13, 0, OpCode::And(Kind::I128, params[0], params[1])),
            //public transactional extFun or(num1:.I128, num2:.I128):(res:.I128);
            10 => just_local_gas_and_mem(13, 0, OpCode::Or(Kind::I128, params[0], params[1])),
            //public transactional extFun xor(num1:.I128, num2:.I128):(res:.I128);
            11 => just_local_gas_and_mem(13, 0, OpCode::Xor(Kind::I128, params[0], params[1])),
            //public transactional extFun not(num1:.I128):(res:.I128);
            12 => just_local_gas_and_mem(13, 0, OpCode::Not(Kind::I128, params[0])),
            //public extFun toData(num:.I128):(res:Data.Data16);
            13 => just_local_gas_and_mem(18, 16, OpCode::ToData(Kind::I128, params[0])),
            //public extFun fromData(data:Data.Data16):(res:.I128);
            14 => just_local_gas_and_mem(18, 0, OpCode::FromData(Kind::I128, params[0])),
            //public extFun hash(num:.I128):(res:Data.Data20);
            //Todo: SysInvokeDoesNotYet have a dynamic gas version
            15 => just_local_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::I128, params)),
            _ => return error(||"External call is not defined")
        })
    }
}