use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Exp};
use sanskrit_common::model::ValueRef;
use crate::bench::opcodes::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;
use crate::bench::opcodes::unpack;
use crate::bench::opcodes::void as void_mod;


//it gets very even:
//30-45 -- which is nothing considering the scope of 56 - 646 (before subtracting the 2 unpacks)
//avg: 40 <-- confirmed by void
// Cost: 40
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TrySuccTest(usize,u8);
impl StructOp for TrySuccTest {
    fn get_fields(&self) -> u8 { self.1}
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let body = OpCode::Unpack(ValueRef((iter*(self.1 as usize)) as u16));
        let success = Exp(alloc.copy_alloc_slice(&[OpCode::Unpack(ValueRef(((1+iter)*(self.1 as usize)) as u16))]).unwrap());
        let fail = Exp(alloc.copy_alloc_slice(&[OpCode::Unpack(ValueRef((iter*(self.1 as usize)) as u16))]).unwrap());
        OpCode::Try(alloc.alloc(body).unwrap(), alloc.alloc(success).unwrap(), alloc.alloc(fail).unwrap())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<StructOpTest<TrySuccTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:StructOpTest<TrySuccTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, fields:u8) -> StructOpTest<TrySuccTest> {StructOpTest(TrySuccTest(rep,fields))}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TrySuccVoidTest(usize);
impl StructOp for TrySuccVoidTest {
    fn get_fields(&self) -> u8 { 0 }
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let body = OpCode::Void;
        let success = Exp(alloc.copy_alloc_slice(&[OpCode::Void]).unwrap());
        let fail = Exp(alloc.copy_alloc_slice(&[OpCode::Void]).unwrap());
        OpCode::Try(alloc.alloc(body).unwrap(), alloc.alloc(success).unwrap(), alloc.alloc(fail).unwrap())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE_VOID: Mutex<BTreeMap<StructOpTest<TrySuccVoidTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure_void(test:StructOpTest<TrySuccVoidTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE_VOID.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test_void(rep:usize) -> StructOpTest<TrySuccVoidTest> {StructOpTest(TrySuccVoidTest(rep))}

pub fn measure_gas(loops:usize) {
    let op = "TrySucc";

    let void_base = measure(test(1000,0), loops) as i128;
    let void = void_mod::measure(void_mod::test(1000), loops) as i128;
    let cor_void_base = void_base - 2*void;
    println!("{}Void{} - {}", op, 0, cor_void_base/1000);

    let base = measure(test(1000,0), loops) as i128;
    let unpack_base = unpack::measure(unpack::test(1000,0), loops) as i128;
    let cor_base = base - 2*unpack_base;
    println!("{}Returns{} - {}", op, 0, cor_base/1000);
    let mut inter = Interpolator::new(cor_base,0);
    let trials = vec![1,4,16,32,64];
    for i in &trials {
        let res = measure(test(1000,*i), loops) as i128;
        let unpack_res = unpack::measure(unpack::test(1000,*i), loops) as i128;
        let cor_res = res - 2*unpack_res;
        println!("{}Returns{} - {}", op, i, cor_res/1000);
        inter.add_measure(cor_res,*i as usize);
    }
    println!("{} - {} + {}*returns", op, cor_base/1000, inter.eval()/1000.0)
}

mod void {
    use super::*;
    #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test_void(0), b).unwrap()}
    #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test_void(500), b).unwrap()}
    #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test_void(1000), b).unwrap()}
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