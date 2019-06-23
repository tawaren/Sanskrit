use sanskrit_common::store::Store;
use sanskrit_common::errors::*;
use model::{OpCode, LitDesc};
use sanskrit_common::model::{Hash, SlicePtr, ValueRef};
use sanskrit_common::arena::{SliceBuilder, HeapArena};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CallResources {
    pub max_gas:u64,
    pub max_manifest_stack: u32,
    pub max_frames: u32,
}

//todo: shall we link statically ?
// compiler currently needs interpreter anyway
pub trait CompilationExternals {
    fn compile_call<'b,'h>(fun_idx:u16, params:SlicePtr<'b,ValueRef>, caller:&Hash,  alloc:&'b HeapArena<'h>) -> Result<Option<(CallResources, OpCode<'b>)>>;
    fn compile_lit<'b,'h>(lit_idx: u16, data:SlicePtr<'b,u8>, caller: &Hash, alloc:&'b HeapArena<'h>) -> Result<(CallResources, OpCode<'b>)>;
}

fn just_gas(gas:u64, code:OpCode) -> (CallResources, OpCode) {
    (CallResources{ max_gas: gas, max_manifest_stack: 0, max_frames: 0 }, code)
}

pub struct Externals;
impl CompilationExternals for Externals {
    //todo: make correct gas etc...
    //todo: shall we give in lit Sizes for better gas calc?? -- None for None lits
    //todo: add logics as shorttuc (for bool & data)
    //      additionally we curently would need an idx for all the different ToU / ToI
    // Todo: Pass in LitSizes if Known
    fn compile_call<'b,'h>(fun_idx: u16, params:SlicePtr<'b,ValueRef>, caller: &Hash,  alloc:&'b HeapArena<'h>) -> Result<Option<(CallResources, OpCode<'b>)>> {
        //toI / toU will be 11
        Ok(Some(match fun_idx {
            0 => return Ok(None),
            //currently we have max 32 Byte:
            1 => just_gas(14, OpCode::Eq(params[0], params[1])),
            //currently we have max 32 Byte:
            2 => just_gas(65, OpCode::Hash(params[0])),
            3 => just_gas(70, OpCode::Derive(params[0], params[1])),
            4 => just_gas(13, OpCode::Lt(params[0], params[1])),
            5 => just_gas(13, OpCode::Lte(params[0], params[1])),
            6 => just_gas(13, OpCode::Gt(params[0], params[1])),
            7 => just_gas(13, OpCode::Gte(params[0], params[1])),
            8 => just_gas(12, OpCode::Add(params[0], params[1])),
            9 => just_gas(12, OpCode::Sub(params[0], params[1])),
            10 => just_gas(17, OpCode::Div(params[0], params[1])),
            11 => just_gas(13, OpCode::Mul(params[0], params[1])),
            12 => just_gas(18, OpCode::ToData(params[0])),
            13 => just_gas(13, OpCode::Lit(alloc.copy_alloc_slice(caller)?)),
            n => unimplemented!("{}", n)
        }))
    }

    fn compile_lit<'b,'h>(lit_idx: u16, data:SlicePtr<'b,u8>, caller: &Hash,  alloc:&'b HeapArena<'h>) -> Result<(CallResources, OpCode<'b>)> {
        Ok(match lit_idx {
            0 => match data.len() {
                1 => just_gas(7, OpCode::SpecialLit(data,LitDesc::U8)),
                2 => just_gas(7, OpCode::SpecialLit(data,LitDesc::U16)),
                4 => just_gas(7, OpCode::SpecialLit(data,LitDesc::U32)),
                8 => just_gas(7, OpCode::SpecialLit(data,LitDesc::U64)),
                16 => just_gas(7, OpCode::SpecialLit(data,LitDesc::U128)),
                s => unimplemented!("0:{}",s)
            }
            1 => match data.len() {
                1 => just_gas(7, OpCode::SpecialLit(data,LitDesc::I8)),
                2 => just_gas(7, OpCode::SpecialLit(data,LitDesc::I16)),
                4 => just_gas(7, OpCode::SpecialLit(data,LitDesc::I32)),
                8 => just_gas(7, OpCode::SpecialLit(data,LitDesc::I64)),
                16 => just_gas(7, OpCode::SpecialLit(data,LitDesc::I128)),
                s => unimplemented!("1:{}",s)
            }
            n => unimplemented!("{}", n)
        })
    }
}