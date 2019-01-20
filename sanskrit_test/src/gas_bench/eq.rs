#[cfg(test)]

//todo: put into the gas meterer
/// Note: Analysis of result has shown that a good metric is
/// (Note Data is not yet considered)
/// 70us + (4us * PrimitiveLeafes) + (6us * AdtLeafes) + (9us * AdtNodes + DataNodes)
/// +TotalData/5 <- this vaires widly hard to define (but necessary to prevent giant data attacks)
/// (5 is really pesimistic, it gets less expensive isf data gets bigger, is about right for 20Byte leafes)
//todo: reevaluate the constant (70) after the calls are lifted
use super::*;
const OP:[Operand;1] = [Operand::Eq];

#[cfg(test)]
mod single {
    use super::*;


    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap(), a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(-12)).unwrap(), a.alloc(Object::I8(-12)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(310)).unwrap(), a.alloc(Object::U16(310)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(-120)).unwrap(), a.alloc(Object::I16(-120)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(3100)).unwrap(), a.alloc(Object::U32(3100)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I32(-1200)).unwrap(), a.alloc(Object::I32(-1200)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(3100)).unwrap(), a.alloc(Object::U64(3100)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(-1200)).unwrap(), a.alloc(Object::I64(-1200)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(310000)).unwrap(), a.alloc(Object::U128(310000)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(-120000)).unwrap(), a.alloc(Object::I128(-120000)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all_ints(b: &mut Bencher) {
        execute_native(b, |a| {
            vec![
                vec![a.alloc(Object::U8(31)).unwrap(), a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap(), a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(-12)).unwrap(), a.alloc(Object::I8(-12)).unwrap()],
                vec![a.alloc(Object::U16(310)).unwrap(), a.alloc(Object::U16(310)).unwrap()],
                vec![a.alloc(Object::I16(-120)).unwrap(), a.alloc(Object::I16(-120)).unwrap()],
                vec![a.alloc(Object::U32(3100)).unwrap(), a.alloc(Object::U32(3100)).unwrap()],
                vec![a.alloc(Object::I32(-1200)).unwrap(), a.alloc(Object::I32(-1200)).unwrap()],
                vec![a.alloc(Object::U64(3100)).unwrap(), a.alloc(Object::U64(3100)).unwrap()],
                vec![a.alloc(Object::I64(-1200)).unwrap(), a.alloc(Object::I64(-1200)).unwrap()],
                vec![a.alloc(Object::U128(310000)).unwrap(), a.alloc(Object::U128(310000)).unwrap()],
                vec![a.alloc(Object::I128(-120000)).unwrap(), a.alloc(Object::I128(-120000)).unwrap()]
            ]
        }, &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap(), a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap(), a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(-12)).unwrap(), a.alloc(Object::I8(-12)).unwrap()],
                vec![a.alloc(Object::U16(310)).unwrap(), a.alloc(Object::U16(310)).unwrap()],
                vec![a.alloc(Object::I16(-120)).unwrap(), a.alloc(Object::I16(-120)).unwrap()],
                vec![a.alloc(Object::U32(3100)).unwrap(), a.alloc(Object::U32(3100)).unwrap()],
                vec![a.alloc(Object::I32(-1200)).unwrap(), a.alloc(Object::I32(-1200)).unwrap()],
                vec![a.alloc(Object::U64(3100)).unwrap(), a.alloc(Object::U64(3100)).unwrap()],
                vec![a.alloc(Object::I64(-1200)).unwrap(), a.alloc(Object::I64(-1200)).unwrap()],
                vec![a.alloc(Object::U128(310000)).unwrap(), a.alloc(Object::U128(310000)).unwrap()],
                vec![a.alloc(Object::I128(-120000)).unwrap(), a.alloc(Object::I128(-120000)).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}


mod wide {
    use super::*;

    fn obj<'a>(alloc: &'a VirtualHeapArena, leaf: Object<'a>, param: u8) -> Ptr<'a, Object<'a>> {
        wide_obj(alloc, leaf, param)
    }

    mod p1 {
        use super::*;

        const PARAM: u8 = 1;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p4 {
        use super::*;

        const PARAM: u8 = 4;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p16 {
        use super::*;

        const PARAM: u8 = 16;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p64 {
        use super::*;

        const PARAM: u8 = 64;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }


    mod p128 {
        use super::*;

        const PARAM: u8 = 128;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }
}

mod deep {
    use super::*;

    fn obj<'a>(alloc: &'a VirtualHeapArena, leaf: Object<'a>, param: u8) -> Ptr<'a, Object<'a>> {
        deep_obj(alloc, leaf, param)
    }

    mod p1 {
        use super::*;

        const PARAM: u8 = 1;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p4 {
        use super::*;

        const PARAM: u8 = 4;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p16 {
        use super::*;

        const PARAM: u8 = 16;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p64 {
        use super::*;

        const PARAM: u8 = 64;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }


    mod p128 {
        use super::*;

        const PARAM: u8 = 128;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }
}


mod tree {
    use super::*;

    fn obj<'a>(alloc: &'a VirtualHeapArena, leaf: Object<'a>, param: u8) -> Ptr<'a, Object<'a>> {
        tree_obj(alloc, leaf, param)
    }

    mod p2 {
        use super::*;

        const PARAM: u8 = 1;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p4 {
        use super::*;

        const PARAM: u8 = 2;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p16 {
        use super::*;

        const PARAM: u8 = 4;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }

    mod p64 {
        use super::*;

        const PARAM: u8 = 6;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }


    mod p128 {
        use super::*;

        const PARAM: u8 = 7;

        #[bench]
        fn bench_u8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i8(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i16(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i32(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i64(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_u128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_i128(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data1(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data10(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data20(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data50(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data100(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data200(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_data_empty(b: &mut Bencher) {
            execute_native(b, |a| vec![vec![obj(a, Object::Adt(0, SlicePtr::empty()), PARAM), obj(a, Object::Adt(0, SlicePtr::empty()), PARAM)]], &OP, true);
        }

        #[bench]
        fn bench_all_int(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)]
                ]
            }, &OP, true);
        }

        #[bench]
        fn bench_all(b: &mut Bencher) {
            execute_native(b, |a| {
                vec![
                    vec![obj(a, Object::U8(31), PARAM), obj(a, Object::U8(31), PARAM)],
                    vec![obj(a, Object::I8(31), PARAM), obj(a, Object::I8(31), PARAM)],
                    vec![obj(a, Object::U16(310), PARAM), obj(a, Object::U16(310), PARAM)],
                    vec![obj(a, Object::I16(310), PARAM), obj(a, Object::I16(310), PARAM)],
                    vec![obj(a, Object::U32(3100), PARAM), obj(a, Object::U32(3100), PARAM)],
                    vec![obj(a, Object::I32(3100), PARAM), obj(a, Object::I32(3100), PARAM)],
                    vec![obj(a, Object::U64(31000), PARAM), obj(a, Object::U64(31000), PARAM)],
                    vec![obj(a, Object::I64(31000), PARAM), obj(a, Object::I64(31000), PARAM)],
                    vec![obj(a, Object::U128(310000), PARAM), obj(a, Object::U128(310000), PARAM)],
                    vec![obj(a, Object::I128(310000), PARAM), obj(a, Object::I128(310000), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap()), PARAM)],
                    vec![obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM), obj(a, Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap()), PARAM)]
                ]
            }, &OP, true);
        }
    }
}