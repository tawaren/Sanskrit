use test::Bencher;
use crate::test_utils::run_ops;
use sanskrit_interpreter::model::{OpCode, Kind, Exp};
use sanskrit_common::model::ValueRef;
use crate::bench::tests::struct_op::{StructOp, StructOpTest};
use sanskrit_common::arena::VirtualHeapArena;

//Result:
// Base: 50 - 10 Ret = 40  (ctr have no influence here)
//  Fiedls: 3.5*Fields
// Rets beeing free is same behaviour as let (probably well cached ret vec (or even register allocated?)
//  Rets: 23*Ret - 25 Ret = -2*Ret
// Res: 40 + 3.5*Fields -- use: 40 + 5*fields (for symmetry with unpack)
//  Note: let has 30 cost, so this is 10 more which fits as switch has to additionally read a field
struct SwitchTest(usize,u8,u8,u8);
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
        for i in 0..self.2 {
            builder2.push(exp);
        }

        OpCode::Switch(ValueRef((iter*self.3 as usize) as u16),builder2.finish())
    }
}


fn test(rep:usize, fields:u8, cases:u8, rets:u8) -> StructOpTest<SwitchTest> {StructOpTest(SwitchTest(rep,fields, cases, rets))}

mod c1 {
    use super::*;
    const CASES:u8 = 1;
    mod f0 {
        use super::*;
        const FIELDS:u8 = 0;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
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
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
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
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f1 {
        use super::*;
        const FIELDS:u8 = 1;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f4 {
        use super::*;
        const FIELDS:u8 = 4;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f16 {
        use super::*;
        const FIELDS:u8 = 16;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }

    mod f64 {
        use super::*;
        const FIELDS:u8 = 64;
        mod r0 {
            use super::*;
            const RET:u8 = 0;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r1 {
            use super::*;
            const RET:u8 = 1;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r4 {
            use super::*;
            const RET:u8 = 4;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r16 {
            use super::*;
            const RET:u8 = 16;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }

        mod r64 {
            use super::*;
            const RET:u8 = 64;
            #[bench] fn bench_0(b: &mut Bencher) {run_ops(test(0, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_500(b: &mut Bencher) {run_ops(test(500, FIELDS,CASES, RET), b).unwrap()}
            #[bench] fn bench_1000(b: &mut Bencher) {run_ops(test(1000, FIELDS,CASES, RET), b).unwrap()}
        }
    }
}