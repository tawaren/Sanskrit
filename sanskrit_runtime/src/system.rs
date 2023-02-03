use sanskrit_interpreter::externals::RuntimeExternals;
use sanskrit_common::store::Store;
use verify::TransactionVerificationContext;
use compute::TransactionExecutionContext;
use TransactionBundle;
use sanskrit_common::encoding::ParserAllocator;
use sanskrit_common::errors::*;



pub trait SystemContext<'c> {
    type RE:RuntimeExternals;
    type S:Store;
    type B:TransactionBundle;
    type VC:TransactionVerificationContext<Self::S, Self::B>;
    type EC:TransactionExecutionContext<Self::S, Self::B>;

    fn parse_bundle<A: ParserAllocator>(data:&[u8], alloc:&'c A) -> Result<Self::B>;
}
