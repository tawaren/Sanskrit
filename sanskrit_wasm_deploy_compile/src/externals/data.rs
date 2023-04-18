use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use externals::External;
use sanskrit_compile::externals::{just_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind};

pub const EXT_DATA:&'static dyn External = &Data;

pub struct Data;
impl External for Data {
    /*
    global external(1) primitive data Data1
    global external(2) primitive data Data2
    global external(4) primitive data Data4
    global external(8) primitive data Data8
    global external(12) primitive data Data12
    global external(16) primitive data Data16
    global external(20) primitive data Data20
    global external(24) primitive data Data24
    global external(28) primitive data Data28
    global external(32) primitive data Data32
    */
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_gas_and_mem((13 + data.len()/50) as u64, data.len() as u64,OpCode::Data(data)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Data(len))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            /*
            global external function eq1(data1:Data1, data2:Data1):(res:Bool)
            global external function eq2(data1:Data2, data2:Data2):(res:Bool)
            global external function eq4(data1:Data4, data2:Data4):(res:Bool)
            global external function eq8(data1:Data8, data2:Data8):(res:Bool)
            global external function eq12(data1:Data12, data2:Data12):(res:Bool)
            global external function eq16(data1:Data16, data2:Data16):(res:Bool)
            global external function eq20(data1:Data20, data2:Data20):(res:Bool)
            global external function eq24(data1:Data24, data2:Data24):(res:Bool)
            global external function eq28(data1:Data28, data2:Data28):(res:Bool)
            global external function eq32(data1:Data32, data2:Data32):(res:Bool)
            */
            //currently we have max 32 Byte:
            x if x < 10 => Ok(just_gas_and_mem(14, 0,OpCode::Eq(Kind::Data,params[0], params[1]))),
            //global external function joinHash(data1:Hash, data2:Hash):(res:Hash)
            10 => Ok(just_gas_and_mem(70, Hash::SIZE as u64, OpCode::SysInvoke(0, params))),
            /*
            global external function hash1(data1:Data1):Hash
            global external function hash2(data1:Data2):Hash
            global external function hash4(data1:Data4):Hash
            global external function hash8(data1:Data8):Hash
            global external function hash12(data1:Data12):Hash
            global external function hash16(data1:Data16):Hash
            global external function hash20(data1:Data20):Hash
            global external function hash24(data1:Data24):Hash
            global external function hash28(data1:Data28):Hash
            global external function hash32(data1:Data32):Hash
            */
            //currently we have max 32 Byte:
            _ =>  Ok(just_gas_and_mem(65, 20, OpCode::TypedSysInvoke(0, Kind::Data, params))),
        }

    }
}
