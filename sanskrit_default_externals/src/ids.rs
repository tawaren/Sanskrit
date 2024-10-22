use alloc::vec::Vec;
use sanskrit_common::model::{Hash, LargeVec, ValueRef};
use sanskrit_compile::externals::CompilationResult;
use sanskrit_chain_code::model::{ValueSchema, OpCode, Kind};
use crate::External;

pub const EXT_IDS:&'static dyn External = &Ids;

pub struct Ids;
impl External for Ids{
    /*
    local external(20) standard data PrivateId
    */
    fn compile_lit(&self, _data_idx: u8, data: &[u8], _caller: &Hash) -> CompilationResult {
        CompilationResult::OpCodeResult(OpCode::Data(LargeVec(data.to_vec())))
    }

    fn get_literal_checker(&self, _data_idx: u8, _len: u16) -> ValueSchema {
        ValueSchema::Data(20)
    }

    fn compile_call(&self, fun_idx: u8, params: Vec<ValueRef>, caller: &Hash) -> CompilationResult {
        match fun_idx {
            //global external function moduleId():(priv:PrivateModuleId)
            0 =>  CompilationResult::OpCodeResult(OpCode::Data(LargeVec(caller.to_vec()))),
            /*
            global external function idFromData(dat:Data20):Id
            global external function idToData(id:Id):Data20
            global external function moduleIdFromData(dat:Data20):ModuleId
            global external function moduleIdToData(id:ModuleId):Data20
            */
            x if x >= 1 && x < 5 => CompilationResult::ReorderResult((&[0]).to_vec()),
            /*
            global external function eqId(id1:Id, id2:Id):Bool
            global external function eqModuleId(id1:ModuleId, id2:ModuleId):Bool
            */
            x if x >= 5 && x < 7 => CompilationResult::OpCodeResult(OpCode::Eq(Kind::Data, params[0], params[1])),

            /*
            global external function privateIdderive(priv:PrivateId, hash:Hash):PrivateId
            global external function idDerive(id:Id, hash:Hash):Id
            global external function privateModuleIdDerive(priv:PrivateModuleId, hash:Hash):PrivateId
            global external function moduleIdDerive(id:ModuleId, hash:Hash):Id
            */
            _ =>  CompilationResult::OpCodeResult(OpCode::SysInvoke(0, params)),

        }
    }
}
