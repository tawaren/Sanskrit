use sanskrit_common::model::{SlicePtr, ValueRef};
use externals::{External, just_gas_and_mem, CompilationResult};
use sanskrit_common::arena::HeapArena;
use model::{OpCode, Kind, LitDesc, ValueSchema};
use sanskrit_common::errors::*;

pub const EXT_U64:&'static dyn External = &U64;

pub struct U64;
impl External for U64{
    //public(create) extType(8) <Copy,Drop,Persist,Value,Unbound> U64;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::U64)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Unsigned(8))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //public extFun eq(num1:.U64, num2:.U64):(res:Bool.Bool);
            0 => just_gas_and_mem(14, 0,OpCode::Eq(Kind::U64,params[0], params[1])),
            //public extFun lt(num1:.U64, num2:.U64):(res:Bool.Bool);
            1 => just_gas_and_mem(13, 0,OpCode::Lt(Kind::U64,params[0], params[1])),
            //public extFun lte(num1:.U64, num2:.U64):(res:Bool.Bool);
            2 => just_gas_and_mem(13, 0,OpCode::Lte(Kind::U64,params[0], params[1])),
            //public extFun gt(num1:.U64, num2:.U64):(res:Bool.Bool);
            3 => just_gas_and_mem(13, 0,OpCode::Gt(Kind::U64,params[0], params[1])),
            //public extFun gte(num1:.U64, num2:.U64):(res:Bool.Bool);
            4 => just_gas_and_mem(13, 0,OpCode::Gte(Kind::U64,params[0], params[1])),
            //public extFun add(num1:.U64, num2:.U64):(res:.U64);
            5 => just_gas_and_mem(12, 0,OpCode::Add(Kind::U64,params[0], params[1])),
            //public extFun sub(num1:.U64, num2:.U64):(res:.U64);
            6 => just_gas_and_mem(12, 0,OpCode::Sub(Kind::U64,params[0], params[1])),
            //public extFun div(num1:.U64, num2:.U64):(res:.U64);
            7 => just_gas_and_mem(17, 0,OpCode::Div(Kind::U64,params[0], params[1])),
            //public extFun mul(num1:.U64, num2:.U64):(res:.U64);
            8 => just_gas_and_mem(13, 0,OpCode::Mul(Kind::U64,params[0], params[1])),
            //public transactional extFun and(num1:.U64, num2:.U64):(res:.U64);
            9 => just_gas_and_mem(13, 0,OpCode::And(Kind::U64,params[0], params[1])),
            //public transactional extFun or(num1:.U64, num2:.U64):(res:.U64);
            10 => just_gas_and_mem(13, 0,OpCode::Or(Kind::U64,params[0], params[1])),
            //public transactional extFun xor(num1:.U64, num2:.U64):(res:.U64);
            11 => just_gas_and_mem(13, 0,OpCode::Xor(Kind::U64,params[0], params[1])),
            //public transactional extFun not(num1:.U64):(res:.U64);
            12 => just_gas_and_mem(13, 0,OpCode::Not(Kind::U64,params[0])),
            //public extFun toData(num:.U64):(res:Data.Data8);
            13 => just_gas_and_mem(18, 8, OpCode::ToData(Kind::U64,params[0])),
            //public extFun fromData(data:Data.Data8):(res:.U64);
            14 => just_gas_and_mem(18, 0, OpCode::FromData(Kind::U64,params[0])),
            //public extFun hash(num:.U64):(res:Data.Data20);
            15 => just_gas_and_mem(65, 20, OpCode::Hash(Kind::U64, params[0])),
            _ => return error(||"External call is not defined")
        })
    }
}