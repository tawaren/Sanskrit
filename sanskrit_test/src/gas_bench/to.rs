#[cfg(test)]

use super::*;

#[cfg(test)]
mod u8 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToU(1);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}


#[cfg(test)]
mod u16 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToU(2);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod u32 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToU(4);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod u64 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToU(8);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod u128 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToU(16);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}


#[cfg(test)]
mod i8 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToI(1);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}


#[cfg(test)]
mod i16 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToI(2);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod i32 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToI(4);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod i64 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToI(8);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }

}

#[cfg(test)]
mod i128 {
    use super::*;
    const OP:[Operand;1] = [Operand::ToI(16);1];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }
}

#[cfg(test)]
mod all_u {
    use super::*;
    const OP:[Operand;5] = [Operand::ToU(1),Operand::ToU(2),Operand::ToU(4),Operand::ToU(8),Operand::ToU(16)];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }
}

#[cfg(test)]
mod all_i {
    use super::*;
    const OP:[Operand;5] = [Operand::ToI(1),Operand::ToI(2),Operand::ToI(4),Operand::ToI(8),Operand::ToI(16)];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }
}

#[cfg(test)]
mod all {
    use super::*;
    const OP:[Operand;10] = [Operand::ToU(1),Operand::ToU(2),Operand::ToU(4),Operand::ToU(8),Operand::ToU(16),Operand::ToI(1),Operand::ToI(2),Operand::ToI(4),Operand::ToI(8),Operand::ToI(16)];

    #[bench]
    fn bench_baseline(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, false);
    }

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i8(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i16(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I16(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U32(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i32(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U8(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i64(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I64(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_u128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::U128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_i128(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::I128(31)).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::U8(31)).unwrap()],
                vec![a.alloc(Object::I8(31)).unwrap()],
                vec![a.alloc(Object::U16(31)).unwrap()],
                vec![a.alloc(Object::I16(31)).unwrap()],
                vec![a.alloc(Object::U32(31)).unwrap()],
                vec![a.alloc(Object::I32(31)).unwrap()],
                vec![a.alloc(Object::U64(31)).unwrap()],
                vec![a.alloc(Object::I64(31)).unwrap()],
                vec![a.alloc(Object::U128(31)).unwrap()],
                vec![a.alloc(Object::I128(31)).unwrap()]
            ]
        }, &OP, true);
    }
}