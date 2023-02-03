use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops};
use sanskrit_interpreter::model::{OpCode, Kind};
use crate::bench::opcodes::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 13 roundup to: 15
// Note if we do that unpack gets cheaper with 14
// Maybe do 15 + 5*fields for unpack
//  Check how far we get of with high field count
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VoidTest(usize);
impl Op for VoidTest {
    fn get_kind(&self) -> Kind { Kind::U8 }
    fn get_params(&self) -> usize { 0 }
    fn get_base_num(&self) -> isize { 0 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, _iter: usize, _:&'b VirtualHeapArena) -> OpCode<'b> { OpCode::Void }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<VoidTest>, u128>> = Mutex::new(BTreeMap::new());
}

pub fn measure(test:OpTest<VoidTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

pub fn test(rep:usize) -> OpTest<VoidTest> {OpTest(VoidTest(rep))}

pub fn measure_gas(loops:usize) {
    println!("Void - {}", measure(test(1000), loops)/1000)
}

#[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0), b).unwrap()}
#[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500), b).unwrap()}
#[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000), b).unwrap()}

