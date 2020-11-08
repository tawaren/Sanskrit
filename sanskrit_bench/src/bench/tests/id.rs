use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops};
use sanskrit_interpreter::model::{OpCode, Kind};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::op::{Op, OpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result: 17-19 -> roundup to: 20
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdTest(usize,Kind,isize);
impl Op for IdTest {
    fn get_kind(&self) -> Kind { self.1 }
    fn get_params(&self) -> usize { 1 }
    fn get_base_num(&self) -> isize { self.2 }
    fn get_repeats(&self) -> usize { self.0 }
    fn build_opcode<'b>(&self, iter: usize, _:&'b VirtualHeapArena)-> OpCode<'b> { OpCode::Id(ValueRef(iter as u16)) }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<OpTest<IdTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:OpTest<IdTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, kind:Kind, num:isize) -> OpTest<IdTest> {OpTest(IdTest(rep,kind,num))}

pub fn measure_gas(loops:usize) {
    let bin_op = "Id";
    for (kind, name, num) in &[
        (Kind::U8,"u8", 100), (Kind::I8,"i8", -50),
        (Kind::U16,"u16", 1000), (Kind::I16,"i16", -500),
        (Kind::U32,"u32", 10000), (Kind::I32,"i32", -5000),
        (Kind::U64,"u64", 100000), (Kind::I64,"i64", -50000),
        (Kind::U128,"u128", 1000000), (Kind::I128,"i128", -500000)
    ] {
        println!("{}_{} - {}", bin_op,name, measure(test(1000,*kind, *num), loops)/1000)
    }
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
