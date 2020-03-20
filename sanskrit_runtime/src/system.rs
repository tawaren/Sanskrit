use sanskrit_common::model::*;
use sanskrit_common::errors::*;
use sanskrit_common::arena::*;
use sanskrit_interpreter::model::RuntimeType;

const SYSTEM_MODULE: Hash = [129, 232, 80, 162, 11, 166, 209, 61, 224, 201, 0, 48, 58, 245, 202, 102, 74, 49, 48, 113];
const ENTRY_OFFSET:u8 = 0;
const CONTEXT_OFFSET:u8 = 1;

const TXT_HASH_OFFSET:u8 = 0;
const CODE_HASH_OFFSET:u8 = 1;
const FULL_HASH_OFFSET:u8 = 2;
const UNIQUE_ID_OFFSET:u8 = 3;

pub fn is_context(typ:Ptr<RuntimeType>) -> bool {
    match *typ {
        RuntimeType::Custom { module:SYSTEM_MODULE, offset:CONTEXT_OFFSET, .. } => true,
        _ => false
    }
}

pub fn is_entry(typ:Ptr<RuntimeType>) -> bool {
    match *typ {
        RuntimeType::Custom { module:SYSTEM_MODULE, offset:ENTRY_OFFSET, .. } => true,
        _ => false
    }
}