use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use externals::External;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};

pub const EXT_U32:&'static dyn External = &U32;

pub struct U32;
impl External for U32{
    //global external(4) primitive data U32;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U32)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Unsigned(4))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.U32, num2:.U32):(res:Bool.Bool);
            0 => just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::U32, params[0], params[1])),
            //global external function lt(num1:.U32, num2:.U32):(res:Bool.Bool);
            1 => just_local_gas_and_mem(13, 0, OpCode::Lt(Kind::U32, params[0], params[1])),
            //global external function lte(num1:.U32, num2:.U32):(res:Bool.Bool);
            2 => just_local_gas_and_mem(13, 0, OpCode::Lte(Kind::U32, params[0], params[1])),
            //global external function gt(num1:.U32, num2:.U32):(res:Bool.Bool);
            3 => just_local_gas_and_mem(13, 0, OpCode::Gt(Kind::U32, params[0], params[1])),
            //global external function gte(num1:.U32, num2:.U32):(res:Bool.Bool);
            4 => just_local_gas_and_mem(13, 0, OpCode::Gte(Kind::U32, params[0], params[1])),
            //global external function add(num1:.U32, num2:.U32):(res:.U32);
            5 => just_local_gas_and_mem(12, 0, OpCode::Add(Kind::U32, params[0], params[1])),
            //global external function sub(num1:.U32, num2:.U32):(res:.U32);
            6 => just_local_gas_and_mem(12, 0, OpCode::Sub(Kind::U32, params[0], params[1])),
            //global external function div(num1:.U32, num2:.U32):(res:.U32);
            7 => just_local_gas_and_mem(17, 0, OpCode::Div(Kind::U32, params[0], params[1])),
            //global external function mul(num1:.U32, num2:.U32):(res:.U32);
            8 => just_local_gas_and_mem(13, 0, OpCode::Mul(Kind::U32, params[0], params[1])),
            //global transactional external function and(num1:.U32, num2:.U32):(res:.U32);
            9 => just_local_gas_and_mem(13, 0, OpCode::And(Kind::U32, params[0], params[1])),
            //global transactional external function or(num1:.U32, num2:.U32):(res:.U32);
            10 => just_local_gas_and_mem(13, 0, OpCode::Or(Kind::U32, params[0], params[1])),
            //global transactional external function xor(num1:.U32, num2:.U32):(res:.U32);
            11 => just_local_gas_and_mem(13, 0, OpCode::Xor(Kind::U32, params[0], params[1])),
            //global transactional external function not(num1:.U32):(res:.U32);
            12 => just_local_gas_and_mem(13, 0, OpCode::Not(Kind::U32, params[0])),
            //global external function toData(num:.U32):(res:Data.Data4);
            13 => just_local_gas_and_mem(18, 4, OpCode::ToData(Kind::U32, params[0])),
            //global external function fromData(data:Data.Data4):(res:.U32);
            14 => just_local_gas_and_mem(18, 0, OpCode::FromData(Kind::U32, params[0])),
            //global external function hash(num:.U32):(res:Data.Data20);
            15 => just_local_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::U32, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
