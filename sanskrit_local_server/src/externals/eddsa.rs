use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use sanskrit_compile::externals::{just_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{ValueSchema, OpCode, Kind};
use externals::External;

pub const EXT_ECDSA:&'static dyn External = &Ecdsa;

pub struct Ecdsa;
impl External for Ecdsa{
    /*
    global external(32) primitive data Pk
    global external(64) primitive data Sig
    */
    fn compile_lit<'b, 'h>(&self, data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match data_idx {
            0 => Ok(just_gas_and_mem(14, 32,OpCode::Data(data))),
            _ => Ok(just_gas_and_mem(15, 64,OpCode::Data(data))),
        }
    }

    fn get_literal_checker<'b, 'h>(&self, data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        match data_idx {
            0 => Ok(ValueSchema::Data(32)),
            _ => Ok(ValueSchema::Data(64)),
        }
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            //global external function derivePublicId(pk:Pk):Id
            0 => Ok(just_gas_and_mem(65, Hash::SIZE as u64, OpCode::TypedSysInvoke(0, Kind::Data, params))),
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
            //Todo: measure this it is guessed based on ethereum gas costs for similar operations
            _ => Ok(just_gas_and_mem(4500, 0, OpCode::SysInvoke(1, params))),

        }
    }
}
