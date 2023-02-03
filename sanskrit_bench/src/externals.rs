use sanskrit_common::errors::*;
use sanskrit_interpreter::externals::{RuntimeExternals, ExecutionInterface};
use sanskrit_common::hashing::HashingDomain;
use sanskrit_common::model::ValueRef;
use sanskrit_interpreter::model::Kind;
use crate::crypto::{plain_hash, join_hash, ecdsa_verify};

pub struct BenchExternals;
impl RuntimeExternals for BenchExternals {
    fn typed_system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Hash
            0 => plain_hash(interface, kind, values[0], tail),
            _ => unreachable!()
        }
    }

    fn system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) -> Result<()>{
        match id {
            //Derive
            0 => join_hash(interface, values[0], values[1], HashingDomain::Derive, tail),
            //EcDsaVerify
            1 => ecdsa_verify(interface, values[0], values[1], values[2], tail),
            _ => unreachable!()
        }
    }
}