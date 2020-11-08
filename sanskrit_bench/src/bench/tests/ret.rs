use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//9-10 + 25-30*fields -> roundto: 15 + 25*fields
// Note: 30 is only for the lower numbers: so we counteract by using base of 15 instead of 10
// Note: with that formula we charge 35 to few with 64 fields correct would be: 25.5
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReturnTest(usize,usize);
impl Op for ReturnTest {
    fn get_kind(&self) -> Kind { Kind::U8}
    fn get_params(&self) -> usize { self.1 }
    fn get_base_num(&self) -> isize { 0 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let mut builder = alloc.slice_builder(self.1).unwrap();
        let base = iter*self.1;
        for i in 0..self.1 {
            builder.push(ValueRef((base + i) as u16));
        }
        OpCode::Return(builder.finish())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<ReturnTest>, u128>> = Mutex::new(BTreeMap::new());
}

pub fn measure(test:OpTest<ReturnTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

pub fn test(rep:usize, returns:u8) -> OpTest<ReturnTest> { OpTest(ReturnTest(rep, returns as usize))}

pub fn measure_gas(loops:usize) {
    let op = "Return";
    let base = measure(test(1000,0), loops) as i128;
    println!("{}Params{} - {}", op, 0, base/1000);
    let mut inter = Interpolator::new(base,0);
    let trials = vec![1,4,16,32,64];
    for i in &trials {
        let res = measure(test(1000,*i as u8), loops) as i128;
        println!("{}Params{} - {}", op, i, res/1000);
        inter.add_measure(res, *i);
    }

    println!("{} - {} + {}*params", op, base/1000, inter.eval()/1000.0)
}

mod f0 {
    use super::*;
    const FIELDS:u8 = 0;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS), b).unwrap()}

}


mod f1 {
    use super::*;
    const FIELDS:u8 = 1;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS), b).unwrap()}
}

mod f4 {
    use super::*;
    const FIELDS:u8 = 4;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS), b).unwrap()}
}

mod f16 {
    use super::*;
    const FIELDS:u8 = 16;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS), b).unwrap()}
}

mod f64 {
    use super::*;
    const FIELDS:u8 = 64;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS), b).unwrap()}
}