use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 13 roundup to: 15
// Note if we do that unpack gets cheaper with 14
// Maybe do 15 + 5*fields for unpack
//  Check how far we get of with high field count
struct VoidTest(usize);
impl Op for VoidTest {
    fn get_kind(&self) -> Kind { Kind::U8 }
    fn get_params(&self) -> usize { 0 }
    fn get_base_num(&self) -> isize { 0 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, _:&'b VirtualHeapArena) -> OpCode<'b> { OpCode::Void }
}

fn test(rep:usize) -> OpTest<VoidTest> {OpTest(VoidTest(rep))}

#[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0), b).unwrap()}
#[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500), b).unwrap()}
#[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000), b).unwrap()}

