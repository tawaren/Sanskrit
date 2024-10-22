use alloc::vec::Vec;
use sanskrit_common::model::{Hash, LargeVec, ValueRef};
use sanskrit_compile::externals::CompilationResult;
use sanskrit_chain_code::model::{ValueSchema, OpCode, Kind, LitDesc};
use crate::External;

pub struct IX{
    size:u8,
    kind:Kind,
    desc:LitDesc
}

pub const EXT_I8:&'static dyn External = &IX{size:1,kind:Kind::I8,desc:LitDesc::I8};
pub const EXT_I16:&'static dyn External = &IX{size:2,kind:Kind::I16,desc:LitDesc::I16};
pub const EXT_I32:&'static dyn External = &IX{size:4,kind:Kind::I32,desc:LitDesc::I32};
pub const EXT_I64:&'static dyn External = &IX{size:8,kind:Kind::I64,desc:LitDesc::I64};
pub const EXT_I128:&'static dyn External = &IX{size:16,kind:Kind::I128,desc:LitDesc::I128};

impl External for IX {
    //global external(?) data I?;
    fn compile_lit(&self, _data_idx: u8, data: &[u8], _caller: &Hash) -> CompilationResult {
        CompilationResult::OpCodeResult(OpCode::SpecialLit(LargeVec(data.to_vec()), self.desc))
    }

    fn get_literal_checker(&self, _data_idx: u8, _len: u16) -> ValueSchema {
        ValueSchema::Signed(self.size)
    }

    fn compile_call(&self, fun_idx: u8, params: Vec<ValueRef>, _caller: &Hash) -> CompilationResult {
        match fun_idx {
            //this is the identity funcntion (used for conversions where bit pattern does not change)
            //global external function eq(num1:.I?, num2:.I?):(res:Bool.Bool);
            0 => CompilationResult::OpCodeResult(OpCode::Eq(self.kind, params[0], params[1])),
            //global external function lt(num1:.I?, num2:.I?):(res:Bool.Bool);
            1 => CompilationResult::OpCodeResult(OpCode::Lt(self.kind, params[0], params[1])),
            //global external function lte(num1:.I?, num2:.I?):(res:Bool.Bool);
            2 => CompilationResult::OpCodeResult(OpCode::Lte(self.kind, params[0], params[1])),
            //global external function gt(num1:.I?, num2:.I?):(res:Bool.Bool);
            3 => CompilationResult::OpCodeResult(OpCode::Gt(self.kind, params[0], params[1])),
            //global external function gte(num1:.I?, num2:.I?):(res:Bool.Bool);
            4 => CompilationResult::OpCodeResult(OpCode::Gte(self.kind, params[0], params[1])),
            //global external function add(num1:.I?, num2:.I?):(res:.I?);
            5 => CompilationResult::OpCodeResult(OpCode::Add(self.kind, params[0], params[1])),
            //global external function sub(num1:.I?, num2:.I?):(res:.I?);
            6 => CompilationResult::OpCodeResult(OpCode::Sub(self.kind, params[0], params[1])),
            //global external function div(num1:.I?, num2:.I?):(res:.I?);
            7 => CompilationResult::OpCodeResult(OpCode::Div(self.kind, params[0], params[1])),
            //global external function mul(num1:.I?, num2:.I?):(res:.I?);
            8 => CompilationResult::OpCodeResult(OpCode::Mul(self.kind, params[0], params[1])),
            //global transactional external function and(num1:.I?, num2:.I?):(res:.I?);
            9 => CompilationResult::OpCodeResult(OpCode::And(self.kind, params[0], params[1])),
            //global transactional external function or(num1:.I?, num2:.I?):(res:.I?);
            10 => CompilationResult::OpCodeResult(OpCode::Or(self.kind, params[0], params[1])),
            //global transactional external function xor(num1:.I?, num2:.I?):(res:.I?);
            11 => CompilationResult::OpCodeResult(OpCode::Xor(self.kind, params[0], params[1])),
            //global transactional external function not(num1:.I?):(res:.I?);
            12 => CompilationResult::OpCodeResult(OpCode::Not(self.kind, params[0])),
            //global external function toData(num:.I?):(res:Data.Data?);
            13 => CompilationResult::OpCodeResult(OpCode::ToData(self.kind, params[0])),
            //global external function fromData(data:Data.Data?):(res:.I?);
            14 => CompilationResult::OpCodeResult(OpCode::FromData(self.kind, params[0])),
            //global external function hash(num:.I?):(res:Data.Data20);
            15 => CompilationResult::OpCodeResult(OpCode::TypedSysInvoke(0, self.kind, params)),
            _ => panic!("External call is not defined")
        }
    }
}
