use sanskrit_common::model::{Hash, SlicePtr, ValueRef, Tag};
use sanskrit_common::arena::HeapArena;
use sanskrit_common::errors::*;
use sanskrit_common::encoding::*;
use externals::External;
use sanskrit_compile::externals::{just_gas_and_mem, CompilationResult};
use sanskrit_interpreter::model::{Entry, ValueSchema, OpCode, Kind, Exp};

pub const EXT_IDS:&'static dyn External = &Ids;

pub struct Ids;
impl External for Ids{
    /*
    extType(20) <Copy,Drop,Persist,Value,Unbound> PrivateId;
    //current external also support PrivateId = project(PrivateId)
    public(create) extType(20) <Copy,Drop,Persist,Value,Unbound> PublicId;
    */
    fn compile_lit<'b, 'h>(&self, _data_idx: u8, data: SlicePtr<'b, u8>, _caller: &[u8; 20], _alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        Ok(just_gas_and_mem(14, 20,OpCode::Data(data)))
    }

    fn get_literal_checker<'b, 'h>(&self, _data_idx: u8, _len: u16, _alloc: &'b HeapArena<'h>) -> Result<ValueSchema<'b>> {
        Ok(ValueSchema::Data(20))
    }

    fn compile_call<'b, 'h>(&self, fun_idx: u8, params: SlicePtr<'b, ValueRef>, caller: &[u8; 20], alloc: &'b HeapArena<'h>) -> Result<CompilationResult<'b>> {
        match fun_idx {
            //public extFun moduleId():(pub:.PrivateId);
            0 =>  Ok(just_gas_and_mem(13, Hash::SIZE as u64, OpCode::Data(alloc.copy_alloc_slice(caller)?))),
            /*
            public extFun privateToPublic(priv:.PrivateId):(pub:.PublicId);
            public extFun dataToPublic(data:Data.Data20):(pub:.PublicId);
            public extFun privateToData(priv:.PrivateId):(data:Data.Data20);
            public extFun publicToData(priv:.PublicId):(data:Data.Data20);
            */
            x if x >= 1 && x < 5 => Ok(CompilationResult::ReorderResult(alloc.copy_alloc_slice(&[0])?)),
            /*
            public extFun eqPub(data1:.PublicId, data2:.PublicId):(res:Bool.Bool);
            public extFun eqPriv(data1:.PrivateId, data2:.PrivateId):(res:Bool.Bool);
            */
            x if x >= 5 && x < 7 =>  Ok(just_gas_and_mem(14, 0,OpCode::Eq(Kind::Data,params[0], params[1]))),

            //public extFun authenticatedMessage[<Unbound> T](value:T):(auth:.Auth[T]);
            x if x == 7 => {
                let mut exp_builder = alloc.slice_builder(2)?;
                exp_builder.push(OpCode::Data(alloc.copy_alloc_slice(caller)?));
                exp_builder.push(OpCode::Pack(Tag(0), alloc.copy_alloc_slice(&[ValueRef(0),ValueRef(params[0].0+1)])?));
                Ok(just_gas_and_mem(/*70+13+15 ~ 100*/100, (Hash::SIZE+(Entry::SIZE * 2)) as u64,OpCode::Let(alloc.alloc(Exp(exp_builder.finish())))))
            },

            /*
            public extFun derivePrivateIdPrivate(priv:.PrivateId, priv2:.PrivateId):(priv:.PrivateId);
            public extFun derivePrivateIdPublic(priv:.PrivateId, pub:.PublicId):(priv:.PrivateId);
            public extFun derivePrivateId1(priv:.PrivateId, data:Data.Data1):(priv:.PrivateId);
            public extFun derivePrivateId2(priv:.PrivateId, data:Data.Data2):(priv:.PrivateId);
            public extFun derivePrivateId4(priv:.PrivateId, data:Data.Data4):(priv:.PrivateId);
            public extFun derivePrivateId8(priv:.PrivateId, data:Data.Data8):(priv:.PrivateId);
            public extFun derivePrivateId12(priv:.PrivateId, data:Data.Data12):(priv:.PrivateId);
            public extFun derivePrivateId16(priv:.PrivateId, data:Data.Data16):(priv:.PrivateId);
            public extFun derivePrivateId20(priv:.PrivateId, data:Data.Data20):(priv:.PrivateId);
            public extFun derivePrivateId24(priv:.PrivateId, data:Data.Data24):(priv:.PrivateId);
            public extFun derivePrivateId28(priv:.PrivateId, data:Data.Data28):(priv:.PrivateId);
            public extFun derivePrivateId32(priv:.PrivateId, data:Data.Data32):(priv:.PrivateId);

            public extFun derivePublicIdPrivate(pup:.PublicId, priv:.PrivateId):(pub:.PublicId);
            public extFun derivePublicIdPublic(pub:.PublicId, pub2:.PublicId):(pub:.PublicId);
            public extFun derivePublicId1(pub:.PublicId, data:Data.Data1):(pub:.PublicId);
            public extFun derivePublicId2(pub:.PublicId, data:Data.Data2):(pub:.PublicId);
            public extFun derivePublicId4(pub:.PublicId, data:Data.Data4):(pub:.PublicId);
            public extFun derivePublicId8(pub:.PublicId, data:Data.Data8):(pub:.PublicId);
            public extFun derivePublicId12(pub:.PublicId, data:Data.Data12):(pub:.PublicId);
            public extFun derivePublicId16(pub:.PublicId, data:Data.Data16):(pub:.PublicId);
            public extFun derivePublicId20(pub:.PublicId, data:Data.Data20):(pub:.PublicId);
            public extFun derivePublicId24(pub:.PublicId, data:Data.Data24):(pub:.PublicId);
            public extFun derivePublicId28(pub:.PublicId, data:Data.Data28):(pub:.PublicId);
            public extFun derivePublicId32(pub:.PublicId, data:Data.Data32):(pub:.PublicId);
            */
            _=> Ok(just_gas_and_mem(70, Hash::SIZE as u64, OpCode::SysInvoke(0, params))),
        }
    }
}