use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::{ValueRef, Tag};
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//42-46 + 10-10.5*fields -> roundup to: 45 + 11*fields
//  Maybe we can go down to 10*fields
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PackTest(usize,usize);
impl Op for PackTest {
    fn get_kind(&self) -> Kind { Kind::U8 }
    fn get_params(&self) -> usize { self.1 }
    fn get_base_num(&self) -> isize { 0 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let mut builder = alloc.slice_builder(self.1).unwrap();
        for i in 0..self.1 {
            builder.push(ValueRef((iter + i) as u16));
        }
        OpCode::Pack(Tag(0), builder.finish())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<PackTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:OpTest<PackTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, fields:u8) -> OpTest<PackTest> { OpTest(PackTest(rep, fields as usize))}

pub fn measure_gas(loops:usize) {
    let bin_op = "Pack";
    let base = measure(test(1000,0), loops) as i128;
    println!("{}Fields{} - {}", bin_op, 0, base/1000);
    let mut inter = Interpolator::new(base,0);
    let trials = vec![1, 4,16,32,64,255];
    for i in &trials {
        let res = measure(test(1000,*i as u8), loops) as i128;
        println!("{}Fields{} - {}", bin_op, i, res/1000);
        inter.add_measure(res, *i);
    }

    println!("{} - {} + {}*fields", bin_op, base/1000, inter.eval()/1000.0)
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