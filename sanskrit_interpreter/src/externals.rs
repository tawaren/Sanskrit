use sanskrit_common::errors::*;
use model::{Entry, Kind};
use sanskrit_common::model::ValueRef;
use sanskrit_common::arena::{HeapStack, VirtualHeapArena};

pub trait ExecutionInterface<'interpreter, 'transaction, 'heap> {
    fn get(&self, idx: usize) -> Result<Entry<'transaction>>;
    fn get_stack(&mut self, tail: bool) -> &mut HeapStack<'interpreter, Entry<'transaction>>;
    fn get_heap(&self) -> &'transaction VirtualHeapArena<'heap>;
    fn process_entry_slice<R: Sized, F: FnOnce(&[u8]) -> R>(kind: Kind, op1: Entry<'transaction>, proc: F) -> R;
}

pub trait RuntimeExternals {
    fn typed_system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, kind:Kind, values: &[ValueRef], tail:bool) -> Result<()>;
    fn system_call<'interpreter, 'transaction:'interpreter, 'heap:'transaction, I:ExecutionInterface<'interpreter, 'transaction, 'heap>>(interface:&mut I, id:u8, values: &[ValueRef], tail:bool) -> Result<()>;
}