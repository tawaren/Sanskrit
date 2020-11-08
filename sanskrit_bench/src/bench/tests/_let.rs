use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Kind, Exp};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;
use crate::bench::tests::unpack;

//Result: 45 + 1.2*returns
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LetTest(usize,u8);
impl StructOp for LetTest {
    fn get_fields(&self) -> u8 { self.1}
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let exp = Exp(alloc.copy_alloc_slice(&[OpCode::Unpack(ValueRef((iter*(self.1 as usize)) as u16))]).unwrap());
        OpCode::Let(alloc.alloc(exp).unwrap())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<StructOpTest<LetTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:StructOpTest<LetTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, fields:u8) -> StructOpTest<LetTest> {StructOpTest(LetTest(rep,fields))}

pub fn measure_gas(loops:usize) {
    let op = "Let";
    let base = measure(test(1000,0), loops) as i128;
    let unpack_base = unpack::measure(unpack::test(1000,0), loops) as i128;
    let cor_base = base - unpack_base;
    println!("{}Returns{} - {}", op, 0, cor_base/1000);
    let mut inter = Interpolator::new(cor_base,0);
    let trials = vec![1,4,16,32,64];
    for i in &trials {
        let res = measure(test(1000,*i as u8), loops) as i128;
        let unpack_res = unpack::measure(unpack::test(1000,*i as u8), loops) as i128;
        let cor_res = res - unpack_res;
        println!("{}Returns{} - {}", op, i, cor_res/1000);
        inter.add_measure(cor_res, *i);
    }
    println!("{} - {} + {}*returns", op, cor_base/1000, inter.eval()/1000.0)
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