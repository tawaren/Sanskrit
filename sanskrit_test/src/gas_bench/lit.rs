#[cfg(test)]

use super::*;

//Note: the object

#[bench]
fn bench_u8(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8]).unwrap(),LitDesc::U8), 1,true);
}

#[bench]
fn bench_i8(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8]).unwrap(),LitDesc::I8), 1,true);
}

#[bench]
fn bench_u16(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::U16), 1,true);
}

#[bench]
fn bench_i16(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::I16), 1,true);
}

#[bench]
fn bench_u32(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::U32), 1,true);
}

#[bench]
fn bench_i32(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::I32), 1,true);
}

#[bench]
fn bench_u64(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::U64), 1,true);
}

#[bench]
fn bench_i64(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::I64), 1,true);
}


#[bench]
fn bench_u128(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::U128), 1,true);
}

#[bench]
fn bench_i128(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::I128), 1,true);
}

#[bench]
fn bench_data1(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_data10(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;10]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_data20(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;20]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_data50(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;50]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_data100(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;100]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_data200(b: &mut Bencher){
    execute_code(b, |a|vec![vec![]], |a,_|OpCode::Lit(a.copy_alloc_slice(&[31u8;200]).unwrap(),LitDesc::Data), 1,true);
}

#[bench]
fn bench_all_int(b: &mut Bencher){
    fn build_op<'a>(a:&'a VirtualHeapArena,num:&mut usize) -> OpCode<'a> {
        *num+=1;
        match *num%10 {
            0 => OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::U8),
            1 => OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::I8),
            2 => OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::U16),
            3 => OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::I16),
            4 => OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::U32),
            5 => OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::I32),
            6 => OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::U64),
            7 => OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::I64),
            8 => OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::U128),
            9 => OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::I128),
            _ => unreachable!()
        }
    }
    let mut num = 0;
    execute_code(b, |_|vec![vec![]], |a,_|build_op(a,&mut num), 1,true);
}

#[bench]
fn bench_all(b: &mut Bencher){
    fn build_op<'a>(a:&'a VirtualHeapArena,num:&mut usize) -> OpCode<'a> {
        *num+=1;
        match *num%16 {
            0 => OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::U8),
            1 => OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::I8),
            2 => OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::U16),
            3 => OpCode::Lit(a.copy_alloc_slice(&[31u8;2]).unwrap(),LitDesc::I16),
            4 => OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::U32),
            5 => OpCode::Lit(a.copy_alloc_slice(&[31u8;4]).unwrap(),LitDesc::I32),
            6 => OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::U64),
            7 => OpCode::Lit(a.copy_alloc_slice(&[31u8;8]).unwrap(),LitDesc::I64),
            8 => OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::U128),
            9 => OpCode::Lit(a.copy_alloc_slice(&[31u8;16]).unwrap(),LitDesc::I128),
            10 => OpCode::Lit(a.copy_alloc_slice(&[31u8;1]).unwrap(),LitDesc::Data),
            11 => OpCode::Lit(a.copy_alloc_slice(&[31u8;10]).unwrap(),LitDesc::Data),
            12 => OpCode::Lit(a.copy_alloc_slice(&[31u8;20]).unwrap(),LitDesc::Data),
            13 => OpCode::Lit(a.copy_alloc_slice(&[31u8;50]).unwrap(),LitDesc::Data),
            14 => OpCode::Lit(a.copy_alloc_slice(&[31u8;100]).unwrap(),LitDesc::Data),
            15 => OpCode::Lit(a.copy_alloc_slice(&[31u8;200]).unwrap(),LitDesc::Data),
            _ => unreachable!()
        }
    }
    let mut num = 0;
    execute_code(b, |_|vec![vec![]], |a,_|build_op(a,&mut num), 1,true);
}