use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops};
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 300
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct JoinHashTest(usize);
impl Op for JoinHashTest {
    fn get_kind(&self) -> Kind { Kind::Data }
    fn get_params(&self) -> usize { 2 }
    fn get_base_num(&self) -> isize { 20 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, alloc:&'b VirtualHeapArena)-> OpCode<'b> {
        OpCode::SysInvoke(0, alloc.copy_alloc_slice(&[ValueRef(iter as u16), ValueRef((iter+1) as u16)]).unwrap())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<JoinHashTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:OpTest<JoinHashTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize) -> OpTest<JoinHashTest> {OpTest(JoinHashTest(rep))}

pub fn measure_gas(loops:usize) {
    println!("JoinHash - {}", measure(test(1000), loops)/1000)
}

#[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0), b).unwrap()}
#[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500), b).unwrap()}
#[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000), b).unwrap()}