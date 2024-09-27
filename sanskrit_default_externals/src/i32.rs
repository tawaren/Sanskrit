use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};
use crate::External;

pub const EXT_I32:&'static dyn External = &I32;

pub struct I32;
impl External for I32{
    //global external(4) data I32;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::I32)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Signed(4))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.I32, num2:.I32):(res:Bool.Bool);
            0 => just_local_gas_and_mem(15, 0, OpCode::Eq(Kind::I32, params[0], params[1])),
            //global external function lt(num1:.I32, num2:.I32):(res:Bool.Bool);
            1 => just_local_gas_and_mem(15, 0, OpCode::Lt(Kind::I32, params[0], params[1])),
            //global external function lte(num1:.I32, num2:.I32):(res:Bool.Bool);
            2 => just_local_gas_and_mem(15, 0, OpCode::Lte(Kind::I32, params[0], params[1])),
            //global external function gt(num1:.I32, num2:.I32):(res:Bool.Bool);
            3 => just_local_gas_and_mem(15, 0, OpCode::Gt(Kind::I32, params[0], params[1])),
            //global external function gte(num1:.I32, num2:.I32):(res:Bool.Bool);
            4 => just_local_gas_and_mem(15, 0, OpCode::Gte(Kind::I32, params[0], params[1])),
            //global external function add(num1:.I32, num2:.I32):(res:.I32);
            5 => just_local_gas_and_mem(15, 0, OpCode::Add(Kind::I32, params[0], params[1])),
            //global external function sub(num1:.I32, num2:.I32):(res:.I32);
            6 => just_local_gas_and_mem(15, 0, OpCode::Sub(Kind::I32, params[0], params[1])),
            //global external function div(num1:.I32, num2:.I32):(res:.I32);
            7 => just_local_gas_and_mem(20, 0, OpCode::Div(Kind::I32, params[0], params[1])),
            //global external function mul(num1:.I32, num2:.I32):(res:.I32);
            8 => just_local_gas_and_mem(15, 0, OpCode::Mul(Kind::I32, params[0], params[1])),
            //global transactional external function and(num1:.I32, num2:.I32):(res:.I32);
            9 => just_local_gas_and_mem(15, 0, OpCode::And(Kind::I32, params[0], params[1])),
            //global transactional external function or(num1:.I32, num2:.I32):(res:.I32);
            10 => just_local_gas_and_mem(15, 0, OpCode::Or(Kind::I32, params[0], params[1])),
            //global transactional external function xor(num1:.I32, num2:.I32):(res:.I32);
            11 => just_local_gas_and_mem(15, 0, OpCode::Xor(Kind::I32, params[0], params[1])),
            //global transactional external function not(num1:.I32):(res:.I32);
            12 => just_local_gas_and_mem(15, 0, OpCode::Not(Kind::I32, params[0])),
            //global external function toData(num:.I32):(res:Data.Data4);
            13 => just_local_gas_and_mem(20, 4, OpCode::ToData(Kind::I32, params[0])),
            //global external function fromData(data:Data.Data4):(res:.I32);
            14 => just_local_gas_and_mem(20, 0, OpCode::FromData(Kind::I32, params[0])),
            //global external function hash(num:.I32):(res:Data.Data20);
            15 => just_local_gas_and_mem(120, 20, OpCode::TypedSysInvoke(0, Kind::I32, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
