use test::Bencher;
use std::sync::Mutex;
use std::collections::BTreeMap;
use crate::test_utils::{bench_ops, measure_ops, Interpolator};
use sanskrit_interpreter::model::{OpCode, Exp};
use sanskrit_common::model::ValueRef;
use crate::bench::opcodes::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;
use crate::bench::opcodes::ret;

//Result:
// Base: 50 - 10 Ret = 40  (ctr have no influence here)
//  Fiedls: 3.5*Fields
// Rets beeing free is same behaviour as let (probably well cached ret vec (or even register allocated?)
//  Rets: 23*Ret - 25 Ret = -2*Ret
// Res: 40 + 3.5*Fields -- use: 40 + 5*fields (for symmetry with unpack)
//  Note: let has 30 cost, so this is 10 more which fits as switch has to additionally read a field
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SwitchTest(usize,u8,u8,u8);
impl StructOp for SwitchTest {
    fn get_fields(&self) -> u8 { self.1}
    fn get_repeats(&self) -> usize {self.0}
    fn build_opcode<'b>(&self, iter:usize, alloc:&'b VirtualHeapArena) -> OpCode<'b> {
        assert!(self.3 <= self.1);
        let mut builder = alloc.slice_builder(self.3 as usize).unwrap();
        for i in 0..self.3 {
            builder.push(ValueRef(i as u16));
        }
        let code = OpCode::Return(builder.finish());
        let exp = alloc.alloc(Exp(alloc.copy_alloc_slice(&[code]).unwrap())).unwrap();

        let mut builder2 = alloc.slice_builder(self.2 as usize).unwrap();
        for _ in 0..self.2 {
            builder2.push(exp);
        }

        OpCode::Switch(ValueRef((iter*self.3 as usize) as u16),builder2.finish())
    }
}

lazy_static! {
    pub static ref MEASURE_CACHE: Mutex<BTreeMap<StructOpTest<SwitchTest>, u128>> = Mutex::new(BTreeMap::new());
}

fn measure(test:StructOpTest<SwitchTest>, loops:usize) -> u128 {
    let mut cache = MEASURE_CACHE.lock().unwrap();
    if !cache.contains_key(&test){
        cache.insert(test, measure_ops(test, loops).unwrap());
    }
    *cache.get(&test).unwrap()
}

fn test(rep:usize, fields:u8, cases:u8, rets:u8) -> StructOpTest<SwitchTest> {StructOpTest(SwitchTest(rep,fields, cases, rets))}

// Todo: add aggregation over cases??
pub fn measure_gas(loops:usize) {
    let op = "Switch";
    let base = measure(test(1000,0, 1,0), loops) as i128;
    let return_base = ret::measure(ret::test(1000,0), loops) as i128;
    let cor_base = base - return_base;
    println!("{}Cases{}Fields{}Returns{} - {}", op, 1, 0, 0, cor_base/1000);
    let cases = vec![2,4,16,32,64];
    for c in &cases{
        let case_res = measure(test(1000,0, *c as u8, 0), loops) as i128;
        let cor_base = case_res - return_base;
        println!("{}Cases{}Fields{}Returns{} - {}", op, *c, 0, 0, cor_base/1000);
        let mut inter_fields = Interpolator::new(cor_base, 0);
        let mut ret_sum = 0.0;
        let trials = vec![1,4,16,32,64];
        for p in &trials {
            let param_res = measure(test(1000,*p as u8, *c as u8, 0), loops) as i128;
            let cor_param_res = param_res - return_base;
            println!("{}Cases{}Fields{}Returns{} - {}", op, c, p, 0, cor_param_res/1000);
            let mut inter_return = Interpolator::new(cor_param_res,0);
            inter_fields.add_measure(cor_param_res, *p);
            for r in trials.iter().filter(|r|*r <= p) {
                let return_res = measure(test(1000,*p as u8, *c as u8, *r as u8), loops) as i128;
                let return_inner_base = ret::measure(ret::test(1000,*r as u8), loops) as i128;
                let cor_return_res = return_res - return_inner_base;
                println!("{}Cases{}Fields{}Returns{} - {}", op, c, p, r, cor_return_res/1000);
                inter_return.add_measure(cor_return_res, *p);
            }
            ret_sum += inter_return.eval();
        }
        let param_extra = inter_fields.eval()/1000.0;
        let return_extra = (ret_sum/(trials.len() as f64))/1000.0;
        println!("{} - {} +{}*fields +{}*returns", op, cor_base/1000, param_extra,return_extra)
    }
}


mod c1 {
    use super::*;
    const CASES:u8 = 1;
    mod f0 {
        use super::*;
        const FIELDS:u8 = 0;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }
}


mod c4 {
    use super::*;
    const CASES:u8 = 4;
    mod f0 {
        use super::*;
        const FIELDS:u8 = 0;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }
}


mod c16 {
    use super::*;
    const CASES:u8 = 16;
    mod f0 {
        use super::*;
        const FIELDS:u8 = 0;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) { bench_ops(test(0, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) { bench_ops(test(500, FIELDS, CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) { bench_ops(test(1000, FIELDS, CASES, RET), b).unwrap()}
        }
    }
}