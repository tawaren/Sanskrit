use sanskrit_interpreter::model::{Kind, OpCode, Entry, Exp, Adt};
use sanskrit_common::model::{ValueRef, SlicePtr, Ptr};
use crate::test_utils::TestCode;
use sanskrit_common::arena::VirtualHeapArena;

pub trait StructOp {
    fn get_fields(&self) -> u8;
    fn get_repeats(&self) -> usize;
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b>;
}

pub struct StructOpTest<T:StructOp>(pub T);
impl<T: StructOp> TestCode for StructOpTest<T> {
    fn get_initials<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> SlicePtr<'l, Entry<'l>> {
        let field_entry = Entry{u8: 0};
        let fields = self.0.get_fields();
        let mut builder = alloc.slice_builder(fields as usize).unwrap();
        for _ in 0..fields {
            builder.push(field_entry)
        }

        let entry = Entry{adt:Adt(0,builder.finish())};
        alloc.copy_alloc_slice(&[entry]).unwrap()
    }

    fn get_code<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> Ptr<'l, Exp<'l>>{
        let repeats = self.0.get_repeats();
        let mut builder = alloc.slice_builder(repeats).unwrap();
        for i in 0..repeats {
            builder.push(self.0.build_opcode(i, alloc))
        }
        alloc.alloc(Exp(builder.finish())).unwrap()
    }
}