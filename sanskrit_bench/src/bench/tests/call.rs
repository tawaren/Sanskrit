use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Kind, Exp};
use sanskrit_common::model::ValueRef;
use sanskrit_common::arena::VirtualHeapArena;
use crate::bench::tests::call_op::{CallOp, CallOpTest};
use crate::bench::tests::ret;

//Result 40 + 12-13*params 1.5-3*returns
//  Round to: 40 + 15*params + 3*returns
//   Recheck if try/let really 0 ret cost or just very low
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CallTest(usize,u8,u8);
impl CallOp for CallTest {
    fn get_kind(&self) -> Kind { Kind::U8 }
    fn get_params(&self) -> usize { self.1 as usize}
    fn get_base_num(&self) -> isize { 0 }
    fn get_repeats(&self) -> usize { self.0 }

    fn build_function<'b>(&self, alloc: &'b VirtualHeapArena) -> Exp<'b> {
        let mut builder = alloc.slice_builder(self.2 as usize).unwrap();
        for i in 0..self.2 {
            builder.push(ValueRef(i as u16));
        }
        Exp(alloc.copy_alloc_slice(&[OpCode::Return(builder.finish())]).unwrap())
    }

    fn build_opcode<'b>(&self, iter: usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        let mut builder = alloc.slice_builder(self.1 as usize).unwrap();
        let base = iter*self.2 as usize;
        for i in 0..self.1 {
            builder.push(ValueRef((base + i as usize) as u16));
        }
        OpCode::Invoke(0, builder.finish())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<CallOpTest<CallTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:CallOpTest<CallTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, params:u8, returns:u8) -> CallOpTest<CallTest> { CallOpTest(CallTest(rep, params, returns))}

pub fn measure_gas(loops:usize) {
    let op = "Call";
    let base = measure(test(1000,0,0), loops) as i128;
    let return_base = ret::measure(ret::test(1000,0), loops) as i128;
    let cor_base = base - return_base;
    println!("{}Params{}Returns{} - {}", op, 0, 0, cor_base/1000);
    let mut inter_param = Interpolator::new(cor_base,0);
    let mut ret_sum = 0.0;
    let trials = vec![1,4,16,32,64];
    for p in &trials {
        let param_res = measure(test(1000,*p as u8, 0), loops) as i128;
        let cor_param_res = param_res - return_base;
        println!("{}Params{}Returns{} - {}", op, p, 0, cor_param_res/1000);
        let mut inter_return = Interpolator::new(cor_param_res,0);
        inter_param.add_measure(cor_param_res,*p);
        for r in trials.iter().filter(|r|*r <= p) {
            let return_res = measure(test(1000,*p as u8, *r as u8), loops) as i128;
            let return_inner_base = ret::measure(ret::test(1000,*r as u8), loops) as i128;
            let cor_return_res = return_res - return_inner_base;
            println!("{}Params{}Returns{} - {}", op, p, r, cor_return_res/1000);
            inter_return.add_measure(cor_return_res, *p);
        }
        ret_sum += inter_return.eval();
    }
    let param_extra = inter_param.eval()/1000.0;
    let return_extra = (ret_sum /(trials.len() as f64))/1000.0;

    println!("{} - {} +{}*params +{}*returns", op, cor_base/1000, param_extra,return_extra)
}

mod p0 {
    use super::*;
    const PARAMS:u8 = 0;
    mod r0 {
        use super::*;
        const RETURNS:u8 = 0;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }
}

mod p1 {
    use super::*;
    const PARAMS:u8 = 1;
    mod r0 {
        use super::*;
        const RETURNS:u8 = 0;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r1 {
        use super::*;
        const RETURNS:u8 = 1;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }
}

mod p4 {
    use super::*;
    const PARAMS:u8 = 4;
    mod r0 {
        use super::*;
        const RETURNS:u8 = 0;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r1 {
        use super::*;
        const RETURNS:u8 = 1;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r4 {
        use super::*;
        const RETURNS:u8 = 4;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }
}

mod p16 {
    use super::*;
    const PARAMS:u8 = 16;
    mod r0 {
        use super::*;
        const RETURNS:u8 = 0;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r1 {
        use super::*;
        const RETURNS:u8 = 1;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r4 {
        use super::*;
        const RETURNS:u8 = 4;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r16 {
        use super::*;
        const RETURNS:u8 = 16;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }
}

mod p64 {
    use super::*;
    const PARAMS:u8 = 64;
    mod r0 {
        use super::*;
        const RETURNS:u8 = 0;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r1 {
        use super::*;
        const RETURNS:u8 = 1;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r4 {
        use super::*;
        const RETURNS:u8 = 4;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r16 {
        use super::*;
        const RETURNS:u8 = 16;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }

    mod r64 {
        use super::*;
        const RETURNS:u8 = 64;
        #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, PARAMS, RETURNS), b).unwrap()}
        #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, PARAMS, RETURNS), b).unwrap()}
    }
}