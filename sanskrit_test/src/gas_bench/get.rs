#[cfg(test)]

use super::*;


fn repeat<'a>(alloc:&'a VirtualHeapArena, obj:Object<'a>, rep:usize) -> SlicePtr<'a,Ptr<'a,Object<'a>>> {
    let mut builder = alloc.slice_builder(rep).unwrap();
    for i in 0..rep {
        builder.push(alloc.alloc(obj).unwrap());
    }
    builder.finish()
}


#[bench]
fn bench_1(b: &mut Bencher){
    execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,repeat(a,Object::U8(31), 1))).unwrap(), ]], |_,ins|OpCode::Get(ins[0],0), 1,true);
}

#[bench]
fn bench_2(b: &mut Bencher){
    execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,repeat(a,Object::U8(31), 2))).unwrap()]], |_,ins|OpCode::Get(ins[0],1), 1,true);
}

#[bench]
fn bench_4(b: &mut Bencher){
    execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,repeat(a,Object::U8(31), 4))).unwrap()]], |_,ins|OpCode::Get(ins[0],3), 1,true);
}

#[bench]
fn bench_8(b: &mut Bencher){
    execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,repeat(a,Object::U8(31), 8))).unwrap()]], |_,ins|OpCode::Get(ins[0],7), 1,true);
}

#[bench]
fn bench_16(b: &mut Bencher){
    execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,repeat(a,Object::U8(31), 16))).unwrap()]], |_,ins|OpCode::Get(ins[0],15), 1,true);
}