use sanskrit_interpreter::model::{Kind, OpCode, Entry, Exp};
use sanskrit_common::model::{SlicePtr, Ptr};
use crate::test_utils::TestCode;
use sanskrit_common::arena::VirtualHeapArena;

pub trait CallOp {
    fn get_kind(&self) -> Kind;
    fn get_params(&self) -> usize;
    //in case of data it is length
    fn get_base_num(&self) -> isize;
    fn get_repeats(&self) -> usize;
    fn build_function<'b>(&self, alloc:&'b VirtualHeapArena) -> Exp<'b>;
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b>;
}

fn gen_entry<'l,'h>(kind:Kind, base_num:isize, alloc:&'l VirtualHeapArena<'h>)->Entry<'l> {
    match kind {
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
        Kind::Data => {
            let mut builder = alloc.slice_builder(base_num as usize).unwrap();
            for i in 0..base_num {
                builder.push( (i % 255) as u8)
            }
            Entry{data: builder.finish()}
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CallOpTest<T: CallOp>(pub T);
impl<T: CallOp> TestCode for CallOpTest<T> {
    fn get_initials<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> SlicePtr<'l, Entry<'l>> {
        let params = self.0.get_params();
        let mut builder = alloc.slice_builder(params).unwrap();
        for _ in 0..params {
            builder.push(gen_entry(self.0.get_kind(), self.0.get_base_num(), alloc))
        }
        builder.finish()
    }

    fn get_code<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> Vec<Ptr<'l, Exp<'l>>>{
        let repeats = self.0.get_repeats();
        let mut builder = alloc.slice_builder(repeats).unwrap();
        for i in 0..repeats {
            builder.push(self.0.build_opcode(i,alloc))
        }
        let fun = self.0.build_function(alloc);
        vec![alloc.alloc(fun).unwrap(), alloc.alloc(Exp(builder.finish())).unwrap()]
    }
}