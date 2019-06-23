use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use sanskrit_common::arena::VirtualHeapArena;
use model::RuntimeType;
use sanskrit_common::capabilities::CapSet;

const SYSTEM_MODULE: Hash = [108, 219, 131, 14, 207, 6, 26, 192, 136, 46, 45, 47, 18, 195, 214, 87, 190, 151, 91, 41];
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
        caps: CapSet::opaque_affine().with_elem(Capability::Consume),
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
        caps: CapSet::from_cap(Capability::Drop),
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