use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use externals::{External, just_gas_and_mem, CompilationResult};
use sanskrit_common::arena::HeapArena;
use model::{OpCode, Kind, LitDesc, ValueSchema};
use sanskrit_common::errors::*;

pub const MODULE:Hash = [174, 99, 175, 166, 24, 255, 139, 69, 144, 248, 210, 246, 99, 118, 237, 71, 157, 112, 176, 158];
pub const EXT_I16:&'static dyn External = &I16;

pub fn check_hash() {
    assert_eq!(format!("{:?}", MODULE), include_str!("../../../sanskrit_test/scripts/out/inti16.hash"));
}

pub struct I16;
impl External for I16 {
    //public(create) extType(2) <Copy,Drop,Persist,Value,Unbound> I16;
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_gas_and_mem(7, 0, OpCode::SpecialLit(data, LitDesc::I16)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Signed(2))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //public extFun eq(num1:.I16, num2:.I16):(res:Bool.Bool);
            0 => just_gas_and_mem(14, 0,OpCode::Eq(Kind::I16,params[0], params[1])),
            //public extFun lt(num1:.I16, num2:.I16):(res:Bool.Bool);
            1 => just_gas_and_mem(13, 0,OpCode::Lt(Kind::I16,params[0], params[1])),
            //public extFun lte(num1:.I16, num2:.I16):(res:Bool.Bool);
            2 => just_gas_and_mem(13, 0,OpCode::Lte(Kind::I16,params[0], params[1])),
            //public extFun gt(num1:.I16, num2:.I16):(res:Bool.Bool);
            3 => just_gas_and_mem(13, 0,OpCode::Gt(Kind::I16,params[0], params[1])),
            //public extFun gte(num1:.I16, num2:.I16):(res:Bool.Bool);
            4 => just_gas_and_mem(13, 0,OpCode::Gte(Kind::I16,params[0], params[1])),
            //public extFun add(num1:.I16, num2:.I16):(res:.I16);
            5 => just_gas_and_mem(12, 0,OpCode::Add(Kind::I16,params[0], params[1])),
            //public extFun sub(num1:.I16, num2:.I16):(res:.I16);
            6 => just_gas_and_mem(12, 0,OpCode::Sub(Kind::I16,params[0], params[1])),
            //public extFun div(num1:.I16, num2:.I16):(res:.I16);
            7 => just_gas_and_mem(17, 0,OpCode::Div(Kind::I16,params[0], params[1])),
            //public extFun mul(num1:.I16, num2:.I16):(res:.I16);
            8 => just_gas_and_mem(13, 0,OpCode::Mul(Kind::I16,params[0], params[1])),
            //public transactional extFun and(num1:.I16, num2:.I16):(res:.I16);
            9 => just_gas_and_mem(13, 0,OpCode::And(Kind::I16,params[0], params[1])),
            //public transactional extFun or(num1:.I16, num2:.I16):(res:.I16);
            10 => just_gas_and_mem(13, 0,OpCode::Or(Kind::I16,params[0], params[1])),
            //public transactional extFun xor(num1:.I16, num2:.I16):(res:.I16);
            11 => just_gas_and_mem(13, 0,OpCode::Xor(Kind::I16,params[0], params[1])),
            //public transactional extFun not(num1:.I16):(res:.I16);
            12 => just_gas_and_mem(13, 0,OpCode::Not(Kind::I16,params[0])),
            //public extFun toData(num:.I16):(res:Data.Data2);
            13 => just_gas_and_mem(18, 2, OpCode::ToData(Kind::I16,params[0])),
            //public extFun fromData(data:Data.Data2):(res:.I16);
            14 => just_gas_and_mem(18, 0, OpCode::FromData(Kind::I16,params[0])),
            //public extFun hash(num:.I16):(res:Data.Data20);
            15 => just_gas_and_mem(65, 20, OpCode::Hash(Kind::I16, params[0])),
            _ => return error(||"External call is not defined")
        })
    }
}