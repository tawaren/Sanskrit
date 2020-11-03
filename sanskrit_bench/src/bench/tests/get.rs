use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 17-20 -> round to: 20
struct GetTest(usize,u8);
impl StructOp for GetTest {
    fn get_fields(&self) -> u8 { self.1}
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, _:&'b VirtualHeapArena) -> OpCode<'b> { OpCode::Get(ValueRef(iter as u16),self.1/2) }
}


fn test(rep:usize, fields:u8) -> StructOpTest<GetTest> {StructOpTest(GetTest(rep, fields))}

mod f1 {
    use super::*;
    const FIELDS:u8 = 1;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}


mod f4 {
    use super::*;
    const FIELDS:u8 = 4;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}
mod f16 {
    use super::*;
    const FIELDS:u8 = 16;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}

mod f64 {
    use super::*;
    const FIELDS:u8 = 64;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}

mod f255 {
    use super::*;
    const FIELDS:u8 = 255;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}