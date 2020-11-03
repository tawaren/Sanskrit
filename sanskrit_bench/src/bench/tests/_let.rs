use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind, Exp};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;

//reevaluate: 50 seems high (switch only has 40) <-- Meet middle 45?
//Result: 38-42 + 4-6*returns
// - 14 + 5,25*fields <-- the nested pack
// = 26 + 0*returns roundto: 30  //Reason: Return Heap probably cached and easy access
struct LetTest(usize,u8);
impl StructOp for LetTest {
    fn get_fields(&self) -> u8 { self.1}
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let exp = Exp(alloc.copy_alloc_slice(&[OpCode::Unpack(ValueRef((iter*(self.1 as usize)) as u16))]).unwrap());
        OpCode::Let(alloc.alloc(exp).unwrap())
    }
}


fn test(rep:usize, fields:u8) -> StructOpTest<LetTest> {StructOpTest(LetTest(rep,fields))}

mod f0 {
    use super::*;
    const FIELDS:u8 = 0;
    #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS), b).unwrap()}
}

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