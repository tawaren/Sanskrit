use alloc::vec::Vec;
use sanskrit_common::model::{Hash, LargeVec, ValueRef};
use sanskrit_compile::externals::CompilationResult;
use sanskrit_chain_code::model::{ValueSchema, OpCode, Kind};
use crate::External;

pub const EXT_ECDSA:&'static dyn External = &Ecdsa;

pub struct Ecdsa;
impl External for Ecdsa{
    /*
    global external(32) primitive data Pk
    global external(64) primitive data Sig
    */
    fn compile_lit(&self, _data_idx: u8, data: &[u8], _caller: &Hash) -> CompilationResult {
            CompilationResult::OpCodeResult(OpCode::Data(LargeVec(data.to_vec())))
    }

    fn get_literal_checker(&self, data_idx: u8, _len: u16) -> ValueSchema {
        match data_idx {
            0 => ValueSchema::Data(32),
            _ => ValueSchema::Data(64),
        }
    }

    fn compile_call(&self, fun_idx: u8, params: Vec<ValueRef>, _caller: &Hash) -> CompilationResult {
        match fun_idx {
            //global external function derivePublicId(pk:Pk):Id
            0 => CompilationResult::OpCodeResult(OpCode::TypedSysInvoke(0, Kind::Data, params)),
            /*
            global external function verify1(msg:Data1, pk:Pk, sig:Sig):(res:Bool)
            global external function verify2(msg:Data2, pk:Pk, sig:Sig):(res:Bool)
            global external function verify4(msg:Data4, pk:Pk, sig:Sig):(res:Bool)
            global external function verify8(msg:Data8, pk:Pk, sig:Sig):(res:Bool)
            global external function verify12(msg:Data12, pk:Pk, sig:Sig):(res:Bool)
            global external function verify16(msg:Data16, pk:Pk, sig:Sig):(res:Bool)
            global external function verify20(msg:Data20, pk:Pk, sig:Sig):(res:Bool)
            global external function verify24(msg:Data24, pk:Pk, sig:Sig):(res:Bool)
            global external function verify28(msg:Data28, pk:Pk, sig:Sig):(res:Bool)
            global external function verify32(msg:Data32, pk:Pk, sig:Sig):(res:Bool)
            */
            _ => CompilationResult::OpCodeResult(OpCode::SysInvoke(1, params)),
        }
    }
}
