use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use externals::External;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};

pub const EXT_U128:&'static dyn External = &U128;

pub struct U128;
impl External for U128{
    //global external(16) primitive data U128;
    fn compile_lit<'b, 'h>(&self, data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match data_idx {
            0 => Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U128))),
            _ => return error(||"External lit is not defined")
        }
    }

    fn get_literal_checker<'b, 'h>(&self, data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match data_idx {
            0 => Ok(ValueSchema::Unsigned(16)),
            _ => return error(||"External lit is not defined")
        }
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.U128, num2:.U128):(res:Bool.Bool);
            0 => just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::U128, params[0], params[1])),
            //global external function lt(num1:.U128, num2:.U128):(res:Bool.Bool);
            1 => just_local_gas_and_mem(13, 0, OpCode::Lt(Kind::U128, params[0], params[1])),
            //global external function lte(num1:.U128, num2:.U128):(res:Bool.Bool);
            2 => just_local_gas_and_mem(13, 0, OpCode::Lte(Kind::U128, params[0], params[1])),
            //global external function gt(num1:.U128, num2:.U128):(res:Bool.Bool);
            3 => just_local_gas_and_mem(13, 0, OpCode::Gt(Kind::U128, params[0], params[1])),
            //global external function gte(num1:.U128, num2:.U128):(res:Bool.Bool);
            4 => just_local_gas_and_mem(13, 0, OpCode::Gte(Kind::U128, params[0], params[1])),
            //global transactional external function add(num1:.U128, num2:.U128):(res:.U128);
            5 => just_local_gas_and_mem(12, 0, OpCode::Add(Kind::U128, params[0], params[1])),
            //global transactional external function sub(num1:.U128, num2:.U128):(res:.U128);
            6 => just_local_gas_and_mem(12, 0, OpCode::Sub(Kind::U128, params[0], params[1])),
            //global transactional external function div(num1:.U128, num2:.U128):(res:.U128);
            7 => just_local_gas_and_mem(17, 0, OpCode::Div(Kind::U128, params[0], params[1])),
            //global transactional external function mul(num1:.U128, num2:.U128):(res:.U128);
            8 => just_local_gas_and_mem(13, 0, OpCode::Mul(Kind::U128, params[0], params[1])),
            //global external function and(num1:.U128, num2:.U128):(res:.U128);
            9 => just_local_gas_and_mem(13, 0, OpCode::And(Kind::U128, params[0], params[1])),
            //global external function or(num1:.U128, num2:.U128):(res:.U128);
            10 => just_local_gas_and_mem(13, 0, OpCode::Or(Kind::U128, params[0], params[1])),
            //global external function xor(num1:.U128, num2:.U128):(res:.U128);
            11 => just_local_gas_and_mem(13, 0, OpCode::Xor(Kind::U128, params[0], params[1])),
            //global external function not(num1:.U128):(res:.U128);
            12 => just_local_gas_and_mem(13, 0, OpCode::Not(Kind::U128, params[0])),
            //global external function toData(num:.U128):(res:Data.Data16);
            13 => just_local_gas_and_mem(18, 16, OpCode::ToData(Kind::U128, params[0])),
            //global external function fromData(data:Data.Data16):(res:.U128);
            14 => just_local_gas_and_mem(18, 0, OpCode::FromData(Kind::U128, params[0])),
            //global external function hash(num:.U128):(res:Data.Data20);
            15 => just_local_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::U128, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
