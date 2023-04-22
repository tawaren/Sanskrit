use sanskrit_common::model::{Hash, SlicePtr, ValueRef, Tag};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use externals::External;
use sanskrit_compile::externals::{just_local_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{Entry, ValueSchema, OpCode, Kind, Exp};

pub const EXT_IDS:&'static dyn External = &Ids;

pub struct Ids;
impl External for Ids{
    /*
    local external(20) standard data PrivateId
    */
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_local_gas_and_mem(14, 20, OpCode::Data(data)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Data(20))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            //global external function moduleId():PrivateModuleId
            0 =>  Ok(just_local_gas_and_mem(13, Hash::SIZE as u64, OpCode::Data(alloc.copy_alloc_slice(caller)?))),
            /*
            global external function idFromData(dat:Data20):Id
            global external function idToData(id:Id):Data20
            global external function moduleIdFromData(dat:Data20):ModuleId
            global external function moduleIdToData(id:ModuleId):Data20
            */
            x if x >= 1 && x < 5 => Ok(CompilationResult::ReorderResult(alloc.copy_alloc_slice(&[0])?)),
            /*
            global external function eqId(id1:Id, id2:Id):Bool
            global external function eqModuleId(id1:ModuleId, id2:ModuleId):Bool
            */
            x if x >= 5 && x < 7 =>  Ok(just_local_gas_and_mem(14, 0, OpCode::Eq(Kind::Data, params[0], params[1]))),

            /*
            global external function privateIdDerive(priv:PrivateId, hash:Hash):PrivateId
            global external function idDerive(id:Id, hash:Hash):Id
            global external function privateModuleIdDerive(priv:PrivateModuleId, hash:Hash):PrivateId
            global external function moduleIdDerive(id:ModuleId, hash:Hash):Id
            */
            _=> Ok(just_local_gas_and_mem(70, Hash::SIZE as u64, OpCode::SysInvoke(0, params))),
        }
    }
}
