use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use externals::{External, just_gas_and_mem, CompilationResult};
use sanskrit_common::arena::HeapArena;
use model::{OpCode, Kind, ValueSchema};
use sanskrit_common::errors::*;

pub const MODULE:Hash = [105, 243, 229, 53, 200, 64, 3, 27, 49, 54, 36, 239, 181, 230, 104, 138, 141, 146, 154, 30];
pub const EXT_DATA:&'static dyn External = &Data;

pub fn check_hash() {
    assert_eq!(format!("{:?}", MODULE), include_str!("../../../sanskrit_test/scripts/out/data.hash"));
}

pub struct Data;
impl External for Data {
    /*
    public(create) extType(1) <Copy,Drop,Persist,Value,Unbound> Data1;
    public(create) extType(2)  <Copy,Drop,Persist,Value,Unbound> Data2;
    public(create) extType(4)  <Copy,Drop,Persist,Value,Unbound> Data4;
    public(create) extType(8) <Copy,Drop,Persist,Value,Unbound> Data8;
    public(create) extType(12) <Copy,Drop,Persist,Value,Unbound> Data12;
    public(create) extType(16) <Copy,Drop,Persist,Value,Unbound> Data16;
    public(create) extType(20) <Copy,Drop,Persist,Value,Unbound> Data20;
    public(create) extType(24) <Copy,Drop,Persist,Value,Unbound> Data24;
    public(create) extType(28) <Copy,Drop,Persist,Value,Unbound> Data28;
    public(create) extType(32) <Copy,Drop,Persist,Value,Unbound> Data32;
    */
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_gas_and_mem((13 + data.len()/50) as u64, data.len() as u64,OpCode::Data(data)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Data(_len))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            /*
            public extFun eq1(data1:.Data1, data2:.Data1):(res:Bool.Bool);
            public extFun eq2(data1:.Data2, data2:.Data2):(res:Bool.Bool);
            public extFun eq4(data1:.Data4, data2:.Data4):(res:Bool.Bool);
            public extFun eq8(data1:.Data8, data2:.Data8):(res:Bool.Bool);
            public extFun eq12(data1:.Data12, data2:.Data12):(res:Bool.Bool);
            public extFun eq16(data1:.Data16, data2:.Data16):(res:Bool.Bool);
            public extFun eq20(data1:.Data20, data2:.Data20):(res:Bool.Bool);
            public extFun eq24(data1:.Data24, data2:.Data24):(res:Bool.Bool);
            public extFun eq28(data1:.Data28, data2:.Data28):(res:Bool.Bool);
            public extFun eq32(data1:.Data32, data2:.Data32):(res:Bool.Bool);
            */
            //currently we have max 32 Byte:
            x if x < 10 =>  Ok(just_gas_and_mem(14, 0,OpCode::Eq(Kind::Data,params[0], params[1]))),
            /*
            public extFun hash1(data1:.Data1):(res:.Data20);
            public extFun hash2(data1:.Data2):(res:.Data20);
            public extFun hash4(data1:.Data4):(res:.Data20);
            public extFun hash8(data1:.Data8):(res:.Data20);
            public extFun hash12(data1:.Data12):(res:.Data20);
            public extFun hash16(data1:.Data16):(res:.Data20);
            public extFun hash20(data1:.Data20):(res:.Data20);
            public extFun hash24(data1:.Data24):(res:.Data20);
            public extFun hash28(data1:.Data28):(res:.Data20);
            public extFun hash32(data1:.Data32):(res:.Data20);
            */
            _ =>  Ok(just_gas_and_mem(65, 20, OpCode::Hash(Kind::Data, params[0]))),
        }

    }
}