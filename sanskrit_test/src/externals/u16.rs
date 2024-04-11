use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};
use externals::External;

pub const EXT_U16:&'static dyn External = &U16;

pub struct U16;
impl External for U16{
    //public(create) extType(2) <Copy,Drop,Persist,Value,Unbound> U16;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U16)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Unsigned(2))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //public extFun eq(num1:.U16, num2:.U16):(res:Bool.Bool);
            0 => just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::U16, params[0], params[1])),
            //public extFun lt(num1:.U16, num2:.U16):(res:Bool.Bool);
            1 => just_local_gas_and_mem(13, 0, OpCode::Lt(Kind::U16, params[0], params[1])),
            //public extFun lte(num1:.U16, num2:.U16):(res:Bool.Bool);
            2 => just_local_gas_and_mem(13, 0, OpCode::Lte(Kind::U16, params[0], params[1])),
            //public extFun gt(num1:.U16, num2:.U16):(res:Bool.Bool);
            3 => just_local_gas_and_mem(13, 0, OpCode::Gt(Kind::U16, params[0], params[1])),
            //public extFun gte(num1:.U16, num2:.U16):(res:Bool.Bool);
            4 => just_local_gas_and_mem(13, 0, OpCode::Gte(Kind::U16, params[0], params[1])),
            //public extFun add(num1:.U16, num2:.U16):(res:.U16);
            5 => just_local_gas_and_mem(12, 0, OpCode::Add(Kind::U16, params[0], params[1])),
            //public extFun sub(num1:.U16, num2:.U16):(res:.U16);
            6 => just_local_gas_and_mem(12, 0, OpCode::Sub(Kind::U16, params[0], params[1])),
            //public extFun div(num1:.U16, num2:.U16):(res:.U16);
            7 => just_local_gas_and_mem(17, 0, OpCode::Div(Kind::U16, params[0], params[1])),
            //public extFun mul(num1:.U16, num2:.U16):(res:.U16);
            8 => just_local_gas_and_mem(13, 0, OpCode::Mul(Kind::U16, params[0], params[1])),
            //public transactional extFun and(num1:.U16, num2:.U16):(res:.U16);
            9 => just_local_gas_and_mem(13, 0, OpCode::And(Kind::U16, params[0], params[1])),
            //public transactional extFun or(num1:.U16, num2:.U16):(res:.U16);
            10 => just_local_gas_and_mem(13, 0, OpCode::Or(Kind::U16, params[0], params[1])),
            //public transactional extFun xor(num1:.U16, num2:.U16):(res:.U16);
            11 => just_local_gas_and_mem(13, 0, OpCode::Xor(Kind::U16, params[0], params[1])),
            //public transactional extFun not(num1:.U16):(res:.U16);
            12 => just_local_gas_and_mem(13, 0, OpCode::Not(Kind::U16, params[0])),
            //public extFun toData(num:.U16):(res:Data.Data2);
            13 => just_local_gas_and_mem(18, 2, OpCode::ToData(Kind::U16, params[0])),
            //public extFun fromData(data:Data.Data2):(res:.U16);
            14 => just_local_gas_and_mem(18, 0, OpCode::FromData(Kind::U16, params[0])),
            //public extFun hash(num:.U16):(res:Data.Data20);
            15 => just_local_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::U16, params)),
            _ => return error(||"External call is not defined")
        })
    }
}