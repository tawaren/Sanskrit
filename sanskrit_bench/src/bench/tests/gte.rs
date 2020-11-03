use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 27-29 -> round to: 30
struct GteTest(usize,Kind,isize);
impl Op for GteTest {
    fn get_kind(&self) -> Kind { self.1 }
    fn get_params(&self) -> usize { 2 }
    fn get_base_num(&self) -> isize { self.2 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize , _:&'b VirtualHeapArena)-> OpCode<'b> { OpCode::Gte(self.1, ValueRef(iter as u16), ValueRef((iter + 1) as u16)) }
}


fn test(rep:usize, kind:Kind, num:isize) -> OpTest<GteTest> { OpTest(GteTest(rep, kind, num))}

mod u8 {
    use super::*;
    const KIND:Kind = Kind::U8;
    const NUM:isize = 100;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i8 {
    use super::*;
    const KIND:Kind = Kind::I8;
    const NUM:isize = -50;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u16 {
    use super::*;
    const KIND:Kind = Kind::U16;
    const NUM:isize = 1000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i16 {
    use super::*;
    const KIND:Kind = Kind::I16;
    const NUM:isize = -500;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u32 {
    use super::*;
    const KIND:Kind = Kind::U32;
    const NUM:isize = 10000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i32 {
    use super::*;
    const KIND:Kind = Kind::I32;
    const NUM:isize = -5000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u64 {
    use super::*;
    const KIND:Kind = Kind::U64;
    const NUM:isize = 100000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i64 {
    use super::*;
    const KIND:Kind = Kind::I64;
    const NUM:isize = -50000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u128 {
    use super::*;
    const KIND:Kind = Kind::U128;
    const NUM:isize = 1000000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i128 {
    use super::*;
    const KIND:Kind = Kind::I128;
    const NUM:isize = -500000;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, KIND, NUM), b).unwrap()}
}
