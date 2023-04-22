use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};
use externals::External;

pub const EXT_U8:&'static dyn External = &U8;

pub struct U8;
impl External for U8{
    //global external(1) data U8;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U8)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Unsigned(1))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.U8, num2:.U8):(res:Bool.Bool);
            0 => just_local_gas_and_mem(15, 0, OpCode::Eq(Kind::U8, params[0], params[1])),
            //global external function lt(num1:.U8, num2:.U8):(res:Bool.Bool);
            1 => just_local_gas_and_mem(15, 0, OpCode::Lt(Kind::U8, params[0], params[1])),
            //global external function lte(num1:.U8, num2:.U8):(res:Bool.Bool);
            2 => just_local_gas_and_mem(15, 0, OpCode::Lte(Kind::U8, params[0], params[1])),
            //global external function gt(num1:.U8, num2:.U8):(res:Bool.Bool);
            3 => just_local_gas_and_mem(15, 0, OpCode::Gt(Kind::U8, params[0], params[1])),
            //global external function gte(num1:.U8, num2:.U8):(res:Bool.Bool);
            4 => just_local_gas_and_mem(15, 0, OpCode::Gte(Kind::U8, params[0], params[1])),
            //global external function add(num1:.U8, num2:.U8):(res:.U8);
            5 => just_local_gas_and_mem(15, 0, OpCode::Add(Kind::U8, params[0], params[1])),
            //global external function sub(num1:.U8, num2:.U8):(res:.U8);
            6 => just_local_gas_and_mem(15, 0, OpCode::Sub(Kind::U8, params[0], params[1])),
            //global external function div(num1:.U8, num2:.U8):(res:.U8);
            7 => just_local_gas_and_mem(20, 0, OpCode::Div(Kind::U8, params[0], params[1])),
            //global external function mul(num1:.U8, num2:.U8):(res:.U8);
            8 => just_local_gas_and_mem(15, 0, OpCode::Mul(Kind::U8, params[0], params[1])),
            //global transactional external function and(num1:.U8, num2:.U8):(res:.U8);
            9 => just_local_gas_and_mem(15, 0, OpCode::And(Kind::U8, params[0], params[1])),
            //global transactional external function or(num1:.U8, num2:.U8):(res:.U8);
            10 => just_local_gas_and_mem(15, 0, OpCode::Or(Kind::U8, params[0], params[1])),
            //global transactional external function xor(num1:.U8, num2:.U8):(res:.U8);
            11 => just_local_gas_and_mem(15, 0, OpCode::Xor(Kind::U8, params[0], params[1])),
            //global transactional external function not(num1:.U8):(res:.U8);
            12 => just_local_gas_and_mem(15, 0, OpCode::Not(Kind::U8, params[0])),
            //global external function toData(num:.U8):(res:Data.Data1);
            13 => just_local_gas_and_mem(20, 1, OpCode::ToData(Kind::U8, params[0])),
            //global external function fromData(data:Data.Data1):(res:.U8);
            14 => just_local_gas_and_mem(20, 0, OpCode::FromData(Kind::U8, params[0])),
            //global external function hash(num:.U8):(res:Data.Data20);
            15 => just_local_gas_and_mem(120, 20, OpCode::TypedSysInvoke(0, Kind::U8, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
