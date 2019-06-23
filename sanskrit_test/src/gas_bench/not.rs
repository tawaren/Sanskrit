#[cfg(test)]

use super::*;
const OP:[Operand;1] = [Operand::Not;1];

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
    execute_native(b, |a|vec![vec![a.alloc(Object::I8(-12)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u16(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::U16(310)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i16(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::I16(-120)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u32(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::U32(1)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i32(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::I32(-1200)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u64(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::U64(3100)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i64(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::I64(-1200)).unwrap()]], &OP, true);
}

#[bench]
fn bench_u128(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::U128(310000)).unwrap()]], &OP, true);
}

#[bench]
fn bench_i128(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::I128(-120000)).unwrap()]], &OP, true);
}

#[bench]
fn bench_data1(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data10(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data20(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data50(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data100(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data200(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
}

#[bench]
fn bench_all_ints(b: &mut Bencher){
    execute_native(b, |a|{
        vec![
            vec![a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::I8(-12)).unwrap()],
            vec![a.alloc(Object::U16(310)).unwrap()],
            vec![a.alloc(Object::I16(-120)).unwrap()],
            vec![a.alloc(Object::U32(1)).unwrap()],
            vec![a.alloc(Object::I32(-1200)).unwrap()],
            vec![a.alloc(Object::U64(3100)).unwrap()],
            vec![a.alloc(Object::I64(-1200)).unwrap()],
            vec![a.alloc(Object::U128(310000)).unwrap()],
            vec![a.alloc(Object::I128(-120000)).unwrap()]
        ]
    }, &OP, true);
}

#[bench]
fn bench_all(b: &mut Bencher){
    execute_native(b, |a|{
        vec![
            vec![a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::U8(31)).unwrap()],
            vec![a.alloc(Object::I8(-12)).unwrap()],
            vec![a.alloc(Object::U16(310)).unwrap()],
            vec![a.alloc(Object::I16(-120)).unwrap()],
            vec![a.alloc(Object::U32(1)).unwrap()],
            vec![a.alloc(Object::I32(-1200)).unwrap()],
            vec![a.alloc(Object::U64(3100)).unwrap()],
            vec![a.alloc(Object::I64(-1200)).unwrap()],
            vec![a.alloc(Object::U128(310000)).unwrap()],
            vec![a.alloc(Object::I128(-120000)).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
        ]
    }, &OP, true);
}