#[cfg(test)]

use super::*;

const OP:[Operand;1] = [Operand::Div;1];

#[bench]
fn bench_baseline(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U8(1)).unwrap(), a.alloc(Object::U8(31)).unwrap()]], &OP, false);
}

#[bench]
fn bench_u8(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U8(1)).unwrap(), a.alloc(Object::U8(31)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i8(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::I8(1)).unwrap(), a.alloc(Object::I8(-12)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u16(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U16(1)).unwrap(), a.alloc(Object::U16(310)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i16(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::I16(1)).unwrap(), a.alloc(Object::I16(-120)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u32(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U32(1)).unwrap(), a.alloc(Object::U32(3100)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i32(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::I32(1)).unwrap(), a.alloc(Object::I32(-1200)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u64(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U64(1)).unwrap(), a.alloc(Object::U64(3100)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i64(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::I64(1)).unwrap(), a.alloc(Object::I64(-1200)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u128(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::U128(1)).unwrap(), a.alloc(Object::U128(310000)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i128(b: &mut Bencher){
    execute_native(b, |a|vec![vec![a.alloc(Object::I128(1)).unwrap(), a.alloc(Object::I128(-120000)).unwrap()]], &OP, true);
}

#[bench]
fn bench_all(b: &mut Bencher){
    execute_native(b, |a|{
        vec![
            vec![a.alloc(Object::U8(1)).unwrap(),a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::U8(1)).unwrap(),a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::I8(1)).unwrap(),a.alloc(Object::I8(-12)).unwrap()],
            vec![a.alloc(Object::U16(1)).unwrap(),a.alloc(Object::U16(310)).unwrap()],
            vec![a.alloc(Object::I16(1)).unwrap(),a.alloc(Object::I16(-120)).unwrap()],
            vec![a.alloc(Object::U32(1)).unwrap(),a.alloc(Object::U32(3100)).unwrap()],
            vec![a.alloc(Object::I32(1)).unwrap(),a.alloc(Object::I32(-1200)).unwrap()],
            vec![a.alloc(Object::U64(1)).unwrap(),a.alloc(Object::U64(3100)).unwrap()],
            vec![a.alloc(Object::I64(1)).unwrap(),a.alloc(Object::I64(-1200)).unwrap()],
            vec![a.alloc(Object::U128(1)).unwrap(),a.alloc(Object::U128(310000)).unwrap()],
            vec![a.alloc(Object::I128(1)).unwrap(),a.alloc(Object::I128(-120000)).unwrap()]
        ]
    }, &OP, true);
}