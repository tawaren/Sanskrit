use alloc::vec::Vec;
use sanskrit_common::model::{Hash, LargeVec, ValueRef};
use sanskrit_common::errors::*;
use sanskrit_compile::externals::CompilationResult;
use sanskrit_chain_code::model::{ValueSchema, OpCode, Kind, LitDesc};
use crate::External;

pub struct UX{
    size:u8,
    kind:Kind,
    desc:LitDesc
}

pub const EXT_U8:&'static dyn External = &UX{size:1,kind:Kind::U8,desc:LitDesc::U8};
pub const EXT_U16:&'static dyn External = &UX{size:2,kind:Kind::U16,desc:LitDesc::U16};
pub const EXT_U32:&'static dyn External = &UX{size:4,kind:Kind::U32,desc:LitDesc::U32};
pub const EXT_U64:&'static dyn External = &UX{size:8,kind:Kind::U64,desc:LitDesc::U64};
pub const EXT_U128:&'static dyn External = &UX{size:16,kind:Kind::U128,desc:LitDesc::U128};

impl External for UX {
    //global external(?) data U?;
    fn compile_lit(&self, _data_idx: u8, data: &[u8], _caller: &Hash) -> Result<CompilationResult> {
        Ok(CompilationResult::OpCodeResult(OpCode::SpecialLit(LargeVec(data.to_vec()), self.desc)))
    }

    fn get_literal_checker(&self, _data_idx: u8, _len: u16) -> Result<ValueSchema> {
        Ok(ValueSchema::Unsigned(self.size))
    }

    fn compile_call(&self, fun_idx: u8, params: Vec<ValueRef>, _caller: &Hash) -> Result<CompilationResult> {
        Ok(match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.U?, num2:.U?):(res:Bool.Bool);
            0 => CompilationResult::OpCodeResult(OpCode::Eq(self.kind, params[0], params[1])),
            //global external function lt(num1:.U?, num2:.U?):(res:Bool.Bool);
            1 => CompilationResult::OpCodeResult(OpCode::Lt(self.kind, params[0], params[1])),
            //global external function lte(num1:.U?, num2:.U?):(res:Bool.Bool);
            2 => CompilationResult::OpCodeResult(OpCode::Lte(self.kind, params[0], params[1])),
            //global external function gt(num1:.U?, num2:.U?):(res:Bool.Bool);
            3 => CompilationResult::OpCodeResult(OpCode::Gt(self.kind, params[0], params[1])),
            //global external function gte(num1:.U?, num2:.U?):(res:Bool.Bool);
            4 => CompilationResult::OpCodeResult(OpCode::Gte(self.kind, params[0], params[1])),
            //global external function add(num1:.U?, num2:.U?):(res:.U?);
            5 => CompilationResult::OpCodeResult(OpCode::Add(self.kind, params[0], params[1])),
            //global external function sub(num1:.U?, num2:.U?):(res:.U?);
            6 => CompilationResult::OpCodeResult(OpCode::Sub(self.kind, params[0], params[1])),
            //global external function div(num1:.U?, num2:.U?):(res:.U?);
            7 => CompilationResult::OpCodeResult(OpCode::Div(self.kind, params[0], params[1])),
            //global external function mul(num1:.U?, num2:.U?):(res:.U?);
            8 => CompilationResult::OpCodeResult(OpCode::Mul(self.kind, params[0], params[1])),
            //global transactional external function and(num1:.U?, num2:.U?):(res:.U?);
            9 => CompilationResult::OpCodeResult(OpCode::And(self.kind, params[0], params[1])),
            //global transactional external function or(num1:.U?, num2:.U?):(res:.U?);
            10 => CompilationResult::OpCodeResult(OpCode::Or(self.kind, params[0], params[1])),
            //global transactional external function xor(num1:.U?, num2:.U?):(res:.U?);
            11 => CompilationResult::OpCodeResult(OpCode::Xor(self.kind, params[0], params[1])),
            //global transactional external function not(num1:.U?):(res:.U?);
            12 => CompilationResult::OpCodeResult(OpCode::Not(self.kind, params[0])),
            //global external function toData(num:.U?):(res:Data.Data?);
            13 => CompilationResult::OpCodeResult(OpCode::ToData(self.kind, params[0])),
            //global external function fromData(data:Data.Data?):(res:.U?);
            14 => CompilationResult::OpCodeResult(OpCode::FromData(self.kind, params[0])),
            //global external function hash(num:.U?):(res:Data.Data20);
            15 => CompilationResult::OpCodeResult(OpCode::TypedSysInvoke(0, self.kind, params)),
            _ => return error(||"External call is not defined")
        })
    }
}
