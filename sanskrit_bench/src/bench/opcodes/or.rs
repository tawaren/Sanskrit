use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::opcodes::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 29-30 -> round to: 30
// Data: 45 + (9*Bytes)/5
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrTest(usize,Kind,isize);
impl Op for OrTest {
    fn get_kind(&self) -> Kind { self.1 }
    fn get_params(&self) -> usize { 2 }
    fn get_base_num(&self) -> isize { self.2 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, _:&'b VirtualHeapArena) -> OpCode<'b> { OpCode::Or(self.1, ValueRef(iter as u16), ValueRef((iter + 1) as u16)) }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<OrTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:OpTest<OrTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, kind:Kind, num:isize) -> OpTest<OrTest> { OpTest(OrTest(rep, kind, num))}

pub fn measure_gas(loops:usize) {
    let bin_op = "Or";
    for (kind, name, num) in &[
        (Kind::U8,"u8", 100), (Kind::I8,"i8", -50),
        (Kind::U16,"u16", 1000), (Kind::I16,"i16", -500),
        (Kind::U32,"u32", 10000), (Kind::I32,"i32", -5000),
        (Kind::U64,"u64", 100000), (Kind::I64,"i64", -50000),
        (Kind::U128,"u128", 1000000), (Kind::I128,"i128", -500000)
    ] {
        println!("{}_{} - {}", bin_op,name, measure(test(1000,*kind, *num), loops)/1000)
    }
    let base = measure(test(1000,Kind::Data, 0), loops) as i128;
    println!("{}Data{} - {}", bin_op, 0, base/1000);
    let mut inter = Interpolator::new(base,0);
    let trials = vec![1,2,4,16,20,50,100,200,500,1000];
    for i in &trials {
        let res = measure(test(1000,Kind::Data, *i as isize), loops) as i128;
        println!("{}Data{} - {}", bin_op, i, res/1000);
        inter.add_measure(res, *i);
    }

    println!("{}Data - {} + {}*bytes", bin_op, base/1000, inter.eval()/1000.0)
}

mod u8 {
    use super::*;
    const KIND:Kind = Kind::U8;
    const NUM:isize = 100;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i8 {
    use super::*;
    const KIND:Kind = Kind::I8;
    const NUM:isize = -50;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u16 {
    use super::*;
    const KIND:Kind = Kind::U16;
    const NUM:isize = 1000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i16 {
    use super::*;
    const KIND:Kind = Kind::I16;
    const NUM:isize = -500;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u32 {
    use super::*;
    const KIND:Kind = Kind::U32;
    const NUM:isize = 10000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i32 {
    use super::*;
    const KIND:Kind = Kind::I32;
    const NUM:isize = -5000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u64 {
    use super::*;
    const KIND:Kind = Kind::U64;
    const NUM:isize = 100000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i64 {
    use super::*;
    const KIND:Kind = Kind::I64;
    const NUM:isize = -50000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod u128 {
    use super::*;
    const KIND:Kind = Kind::U128;
    const NUM:isize = 1000000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod i128 {
    use super::*;
    const KIND:Kind = Kind::I128;
    const NUM:isize = -500000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, NUM), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, NUM), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, NUM), b).unwrap()}
}

mod data1 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 1;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data4 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 4;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data16 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 16;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data20 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 20;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data50 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 50;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data100 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 100;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data200 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 200;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data500 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 500;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}

mod data1000 {
    use super::*;
    const KIND:Kind = Kind::Data;
    const LEN:isize = 1000;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, KIND, LEN), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, KIND, LEN), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, KIND, LEN), b).unwrap()}
}