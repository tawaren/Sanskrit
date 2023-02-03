#![feature(test)]
#![feature(associated_type_defaults)]

pub mod bench;
pub mod externals;
pub mod crypto;

extern crate test;
#[macro_use]
extern crate lazy_static;


mod test_utils {
    use test::Bencher;
    use sanskrit_interpreter::model::{Exp, Entry};
    use sanskrit_interpreter::interpreter::{ExecutionContext, Frame};
    use sanskrit_common::arena::{Heap, VirtualHeapArena};
    use sanskrit_common::model::{SlicePtr, Ptr};
    use sanskrit_common::errors::*;
    use crate::externals::BenchExternals;
    use std::time::Instant;

    pub struct Interpolator{
        base:i128,
        base_parts:usize,
        sum:i128,
        parts:usize
    }

    impl Interpolator {
        pub fn new(base:i128, base_parts:usize) -> Interpolator {
            Interpolator {
                base,
                base_parts,
                sum: 0,
                parts: 0
            }
        }

        pub fn add_measure(&mut self, value:i128, parts:usize){
            assert!(parts > self.base_parts);
            self.sum += value - self.base;
            self.parts += parts - self.base_parts;
        }

        pub fn eval(self) -> f64 {
            self.sum as f64/(self.parts as f64)
        }
    }

    pub trait TestCode {
        fn get_initials<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> SlicePtr<'l, Entry<'l>>;
        fn get_code<'l,'h>(&self, alloc:&'l VirtualHeapArena<'h>) -> Vec<Ptr<'l, Exp<'l>>>;
    }

    pub fn measure_ops<T:TestCode>(t:T, loops:usize) -> Result<u128> {
        let heap = Heap::new(512000000, 2.0);
        let alloc = heap.new_virtual_arena(128000000);
        let functions = t.get_code(&alloc);
        let initials = t.get_initials(&alloc);
        let arena = heap.new_arena(64000000);
        let heap_alloc = heap.new_virtual_arena(64000000);

        let mut sum = 0;

        for _ in 0..loops {
            let tmp_arena = arena.temp_arena();
            let local_heap = heap_alloc.temp_arena().unwrap();
            let mut frames = tmp_arena.alloc_stack::<Frame>(256);
            let mut stack = tmp_arena.alloc_stack::<Entry>(1024*64);
            let mut ret_stack = tmp_arena.alloc_stack::<Entry>(256);
            for v in initials.iter() { stack.push(*v).unwrap(); }

            let inst = Instant::now();
            ExecutionContext::interpret::<BenchExternals>(&functions, &mut stack, &mut frames, &mut ret_stack, &local_heap).unwrap();
            sum += inst.elapsed().as_nanos()
        }

        Ok(sum/(loops as u128))
    }

    pub fn bench_ops<T:TestCode>(t:T, b: &mut Bencher) -> Result<()> {
        let heap = Heap::new(512000000, 2.0);
        let alloc = heap.new_virtual_arena(128000000);
        let functions = t.get_code(&alloc);
        let initials = t.get_initials(&alloc);
        let arena = heap.new_arena(64000000);
        let heap_alloc = heap.new_virtual_arena(64000000);
        b.iter(||{
            let tmp_arena = arena.temp_arena();
            let local_heap = heap_alloc.temp_arena().unwrap();
            let mut frames = tmp_arena.alloc_stack::<Frame>(256);
            let mut stack = tmp_arena.alloc_stack::<Entry>(1024*64);
            let mut ret_stack = tmp_arena.alloc_stack::<Entry>(256);
            for v in initials.iter() { stack.push(*v).unwrap(); }
            ExecutionContext::interpret::<BenchExternals>(&functions, &mut stack, &mut frames, &mut ret_stack, &local_heap).unwrap();
        });
        Ok(())
    }

}

use crate::bench::opcodes::*;
use crate::bench::validate::limit_tests::benchs::*;

fn main() {
    /*add::measure_gas(10000);
    sub::measure_gas(10000);
    mul::measure_gas(10000);
    div::measure_gas(10000);
    and::measure_gas(10000);
    or::measure_gas(10000);
    not::measure_gas(10000);
    xor::measure_gas(10000);
    eq::measure_gas(10000);
    gt::measure_gas(10000);
    gte::measure_gas(10000);
    lt::measure_gas(10000);
    lte::measure_gas(10000);
    void::measure_gas(10000);
    unpack::measure_gas(10000);
    _let::measure_gas(10000);
    try_succ::measure_gas(10000);
    try_fail::measure_gas(10000);
    ret::measure_gas(10000);
    call::measure_gas(10000);
    get::measure_gas(10000);
    pack::measure_gas(10000);
    join_hash::measure_gas(10000);
    plain_hash::measure_gas(10000);
    switch::measure_gas(1000);*/


    bench_01_first_bench(4000);
    bench_02_first_bench(2000);
    bench_04_first_bench(1000);
    bench_08_first_bench(1000);
    bench_10_first_bench(1000);
    bench_20_first_bench(1000);
    bench_40_first_bench(1000);
    bench_01_last_bench(2000);
    bench_02_last_bench(1000);
    bench_04_last_bench(1000);
    bench_08_last_bench(1000);
    bench_10_last_bench(1000);
    bench_20_last_bench(500);
    bench_40_last_bench(250);

}