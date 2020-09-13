use sanskrit_common::model::*;
use sanskrit_interpreter::model::RuntimeType;


pub trait System {
    fn system_module(&self) -> Hash;
    fn entry_offset(&self) -> u8;
    fn context_offset(&self) -> u8;

    fn txt_hash_offset(&self) -> u8;
    fn code_hash_offset(&self) -> u8;
    fn full_hash_offset(&self) -> u8;
    fn unique_id_offset(&self) -> u8;

    fn is_context(&self,typ:Ptr<RuntimeType>) -> bool {
        match *typ {
            RuntimeType::Custom { module, offset, .. } => module == self.system_module() && offset == self.context_offset(),
            _ => false
        }
    }

    fn is_entry(&self,typ:Ptr<RuntimeType>) -> bool {
        match *typ {
            RuntimeType::Custom { module, offset, .. } => module == self.system_module() && offset == self.entry_offset(),
            _ => false
        }
    }
}
