use sanskrit_common::model::{SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use externals::External;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind, LitDesc};

pub const EXT_I8:&'static dyn External = &I8;

pub struct I8;
impl External for I8 {
    //global external(1) primitive data I8;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::I8)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Signed(1))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.I8, num2:.I8):(res:Bool.Bool);
            0 => just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::I8, params[0], params[1])),
            //global external function lt(num1:.I8, num2:.I8):(res:Bool.Bool);
            1 => just_local_gas_and_mem(13, 0, OpCode::Lt(Kind::I8, params[0], params[1])),
            //global external function lte(num1:.I8, num2:.I8):(res:Bool.Bool);
            2 => just_local_gas_and_mem(13, 0, OpCode::Lte(Kind::I8, params[0], params[1])),
            //global external function gt(num1:.I8, num2:.I8):(res:Bool.Bool);
            3 => just_local_gas_and_mem(13, 0, OpCode::Gt(Kind::I8, params[0], params[1])),
            //global external function gte(num1:.I8, num2:.I8):(res:Bool.Bool);
            4 => just_local_gas_and_mem(13, 0, OpCode::Gte(Kind::I8, params[0], params[1])),
            //global external function add(num1:.I8, num2:.I8):(res:.I8);
            5 => just_local_gas_and_mem(12, 0, OpCode::Add(Kind::I8, params[0], params[1])),
            //global external function sub(num1:.I8, num2:.I8):(res:.I8);
            6 => just_local_gas_and_mem(12, 0, OpCode::Sub(Kind::I8, params[0], params[1])),
            //global external function div(num1:.I8, num2:.I8):(res:.I8);
            7 => just_local_gas_and_mem(17, 0, OpCode::Div(Kind::I8, params[0], params[1])),
            //global external function mul(num1:.I8, num2:.I8):(res:.I8);
            8 => just_local_gas_and_mem(13, 0, OpCode::Mul(Kind::I8, params[0], params[1])),
            //global transactional external function and(num1:.I8, num2:.I8):(res:.I8);
            9 => just_local_gas_and_mem(13, 0, OpCode::And(Kind::I8, params[0], params[1])),
            //global transactional external function or(num1:.I8, num2:.I8):(res:.I8);
            10 => just_local_gas_and_mem(13, 0, OpCode::Or(Kind::I8, params[0], params[1])),
            //global transactional external function xor(num1:.I8, num2:.I8):(res:.I8);
            11 => just_local_gas_and_mem(13, 0, OpCode::Xor(Kind::I8, params[0], params[1])),
            //global transactional external function not(num1:.I8):(res:.I8);
            12 => just_local_gas_and_mem(13, 0, OpCode::Not(Kind::I8, params[0])),
            //global external function toData(num:.I8):(res:Data.Data1);
            13 => just_local_gas_and_mem(18, 1, OpCode::ToData(Kind::I8, params[0])),
            //global external function fromData(data:Data.Data1):(res:.I8);
            14 => just_local_gas_and_mem(18, 0, OpCode::FromData(Kind::I8, params[0])),
            //global external function hash(num:.I8):(res:Data.Data20);
            15 => just_local_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::I8, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
