use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::{ValueRef, Tag};
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//9-10 + 25-30*fields -> roundto: 15 + 25*fields
// Note: 30 is only for the lower numbers: so we counteract by using base of 15 instead of 10
// Note: with that formula we charge 35 to few with 64 fields correct would be: 25.5
struct ReturnTest(usize,Kind,isize,usize);
impl Op for ReturnTest {
    fn get_kind(&self) -> Kind { self.1 }
    fn get_params(&self) -> usize { self.3 }
    fn get_base_num(&self) -> isize { self.2 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let mut builder = alloc.slice_builder(self.3).unwrap();
        let base = iter*self.3;
        for i in 0..self.3 {
            builder.push(ValueRef((base + i) as u16));
        }
        OpCode::Return(builder.finish())
    }
}

fn test(rep:usize, kind:Kind,  num:isize, fields:u8) -> OpTest<ReturnTest> { OpTest(ReturnTest(rep, kind, num, fields as usize))}

mod f0 {
    use super::*;
    const FIELDS:u8 = 0;
    mod u8 {
        use super::*;
        const KIND:Kind = Kind::U8;
        const NUM:isize = 100;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i8 {
        use super::*;
        const KIND:Kind = Kind::I8;
        const NUM:isize = -50;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u16 {
        use super::*;
        const KIND:Kind = Kind::U16;
        const NUM:isize = 1000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i16 {
        use super::*;
        const KIND:Kind = Kind::I16;
        const NUM:isize = -500;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u32 {
        use super::*;
        const KIND:Kind = Kind::U32;
        const NUM:isize = 10000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i32 {
        use super::*;
        const KIND:Kind = Kind::I32;
        const NUM:isize = -5000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u64 {
        use super::*;
        const KIND:Kind = Kind::U64;
        const NUM:isize = 100000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i64 {
        use super::*;
        const KIND:Kind = Kind::I64;
        const NUM:isize = -50000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u128 {
        use super::*;
        const KIND:Kind = Kind::U128;
        const NUM:isize = 1000000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i128 {
        use super::*;
        const KIND:Kind = Kind::I128;
        const NUM:isize = -500000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }
}


mod f1 {
    use super::*;
    const FIELDS:u8 = 1;
    mod u8 {
        use super::*;
        const KIND:Kind = Kind::U8;
        const NUM:isize = 100;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i8 {
        use super::*;
        const KIND:Kind = Kind::I8;
        const NUM:isize = -50;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u16 {
        use super::*;
        const KIND:Kind = Kind::U16;
        const NUM:isize = 1000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i16 {
        use super::*;
        const KIND:Kind = Kind::I16;
        const NUM:isize = -500;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u32 {
        use super::*;
        const KIND:Kind = Kind::U32;
        const NUM:isize = 10000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i32 {
        use super::*;
        const KIND:Kind = Kind::I32;
        const NUM:isize = -5000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u64 {
        use super::*;
        const KIND:Kind = Kind::U64;
        const NUM:isize = 100000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i64 {
        use super::*;
        const KIND:Kind = Kind::I64;
        const NUM:isize = -50000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u128 {
        use super::*;
        const KIND:Kind = Kind::U128;
        const NUM:isize = 1000000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i128 {
        use super::*;
        const KIND:Kind = Kind::I128;
        const NUM:isize = -500000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }
}

mod f4 {
    use super::*;
    const FIELDS:u8 = 4;
    mod u8 {
        use super::*;
        const KIND:Kind = Kind::U8;
        const NUM:isize = 100;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i8 {
        use super::*;
        const KIND:Kind = Kind::I8;
        const NUM:isize = -50;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u16 {
        use super::*;
        const KIND:Kind = Kind::U16;
        const NUM:isize = 1000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i16 {
        use super::*;
        const KIND:Kind = Kind::I16;
        const NUM:isize = -500;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u32 {
        use super::*;
        const KIND:Kind = Kind::U32;
        const NUM:isize = 10000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i32 {
        use super::*;
        const KIND:Kind = Kind::I32;
        const NUM:isize = -5000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u64 {
        use super::*;
        const KIND:Kind = Kind::U64;
        const NUM:isize = 100000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i64 {
        use super::*;
        const KIND:Kind = Kind::I64;
        const NUM:isize = -50000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u128 {
        use super::*;
        const KIND:Kind = Kind::U128;
        const NUM:isize = 1000000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i128 {
        use super::*;
        const KIND:Kind = Kind::I128;
        const NUM:isize = -500000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }
}

mod f16 {
    use super::*;
    const FIELDS:u8 = 16;
    mod u8 {
        use super::*;
        const KIND:Kind = Kind::U8;
        const NUM:isize = 100;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i8 {
        use super::*;
        const KIND:Kind = Kind::I8;
        const NUM:isize = -50;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u16 {
        use super::*;
        const KIND:Kind = Kind::U16;
        const NUM:isize = 1000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i16 {
        use super::*;
        const KIND:Kind = Kind::I16;
        const NUM:isize = -500;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u32 {
        use super::*;
        const KIND:Kind = Kind::U32;
        const NUM:isize = 10000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i32 {
        use super::*;
        const KIND:Kind = Kind::I32;
        const NUM:isize = -5000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u64 {
        use super::*;
        const KIND:Kind = Kind::U64;
        const NUM:isize = 100000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i64 {
        use super::*;
        const KIND:Kind = Kind::I64;
        const NUM:isize = -50000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u128 {
        use super::*;
        const KIND:Kind = Kind::U128;
        const NUM:isize = 1000000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i128 {
        use super::*;
        const KIND:Kind = Kind::I128;
        const NUM:isize = -500000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }
}

mod f64 {
    use super::*;
    const FIELDS:u8 = 64;
    mod u8 {
        use super::*;
        const KIND:Kind = Kind::U8;
        const NUM:isize = 100;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i8 {
        use super::*;
        const KIND:Kind = Kind::I8;
        const NUM:isize = -50;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u16 {
        use super::*;
        const KIND:Kind = Kind::U16;
        const NUM:isize = 1000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i16 {
        use super::*;
        const KIND:Kind = Kind::I16;
        const NUM:isize = -500;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u32 {
        use super::*;
        const KIND:Kind = Kind::U32;
        const NUM:isize = 10000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i32 {
        use super::*;
        const KIND:Kind = Kind::I32;
        const NUM:isize = -5000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u64 {
        use super::*;
        const KIND:Kind = Kind::U64;
        const NUM:isize = 100000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i64 {
        use super::*;
        const KIND:Kind = Kind::I64;
        const NUM:isize = -50000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod u128 {
        use super::*;
        const KIND:Kind = Kind::U128;
        const NUM:isize = 1000000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }

    mod i128 {
        use super::*;
        const KIND:Kind = Kind::I128;
        const NUM:isize = -500000;
        #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM, FIELDS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM, FIELDS), b).unwrap()}
    }
}