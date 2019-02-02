use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use sanskrit_common::arena::VirtualHeapArena;
use model::RuntimeType;
use sanskrit_common::capabilities::CapSet;

const SYSTEM_MODULE: Hash = [156, 42, 226, 205, 167, 95, 204, 175, 157, 192, 59, 231, 2, 220, 13, 79, 105, 8, 180, 111];

const ENTRY_OFFSET:u8 = 0;
const SINGLETON_OFFSET:u8 = 1;
const ACCOUNT_OFFSET:u8 = 2;
const CONTEXT_OFFSET:u8 = 3;

const TXT_HASH_OFFSET:u8 = 0;
const CODE_HASH_OFFSET:u8 = 1;
const FULL_HASH_OFFSET:u8 = 2;
const UNIQUE_ID_OFFSET:u8 = 3;

pub fn singleton_type<'a,'h>(alloc:&'a VirtualHeapArena<'h>, n:Ptr<'a, RuntimeType<'a>>) -> Result<RuntimeType<'a>> {
    Ok(RuntimeType::Custom {
        caps: CapSet::opaque_affine().with_elem(NativeCap::Consume),
        module: SYSTEM_MODULE,
        applies: alloc.copy_alloc_slice(&[n])?,
        offset: SINGLETON_OFFSET
    })
}

pub fn account_type<'a,'h>(alloc:&'a VirtualHeapArena<'h>, n:Ptr<'a, RuntimeType<'a>>) -> Result<RuntimeType<'a>> {
    Ok(RuntimeType::Custom {
        caps: CapSet::local(),
        module: SYSTEM_MODULE,
        applies: alloc.copy_alloc_slice(&[n])?,
        offset: ACCOUNT_OFFSET
    })
}

pub fn context_type<'a,'h>() -> RuntimeType<'a> {
    RuntimeType::Custom {
        caps: CapSet::from_cap(NativeCap::Drop),
        module: SYSTEM_MODULE,
        applies: SlicePtr::empty(),
        offset: CONTEXT_OFFSET
    }
}

pub fn is_entry(typ:Ptr<RuntimeType>) -> bool {
    match *typ {
        RuntimeType::Custom { module:SYSTEM_MODULE, offset:ENTRY_OFFSET, .. } => true,
        _ => false
    }
}