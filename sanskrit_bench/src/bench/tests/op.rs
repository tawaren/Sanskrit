use sanskrit_interpreter::model::{Kind, OpCode, Entry, Exp};
use sanskrit_common::model::{ValueRef, SlicePtr, Ptr};
use crate::test_utils::TestCode;
use sanskrit_common::arena::VirtualHeapArena;

pub trait Op {
    fn get_kind(&self) -> Kind;
    fn get_params(&self) -> usize;
    fn get_base_num(&self) -> isize;
    fn get_repeats(&self) -> usize;
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b>;
}

pub struct OpTest<T: Op>(pub T);
impl<T: Op> TestCode for OpTest<T> {
    fn get_initials<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> SlicePtr<'l, Entry<'l>> {
        let base_num = self.0.get_base_num();
        let entry = match self.0.get_kind() {
            Kind::I8 => Entry{i8: base_num as i8},
            Kind::U8 => Entry{u8: base_num as u8},
            Kind::I16 => Entry{i16: base_num as i16},
            Kind::U16 => Entry{u16: base_num as u16},
            Kind::I32 => Entry{i32: base_num as i32},
            Kind::U32 => Entry{u32: base_num as u32},
            Kind::I64 => Entry{i64: base_num as i64},
            Kind::U64 => Entry{u64: base_num as u64},
            Kind::I128 => Entry{i128: base_num as i128},
            Kind::U128 => Entry{u128: base_num as u128},
            Kind::Data => unreachable!()
        };
        let params = self.0.get_params();
        let mut builder = alloc.slice_builder(params).unwrap();
        for i in 0..params {
            builder.push(entry)
        }
        builder.finish()
    }

    fn get_code<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> Ptr<'l, Exp<'l>>{
        let repeats = self.0.get_repeats();
        let mut builder = alloc.slice_builder(repeats).unwrap();
        for i in 0..repeats {
            builder.push(self.0.build_opcode(i,alloc))
        }
        alloc.alloc(Exp(builder.finish())).unwrap()
    }
}