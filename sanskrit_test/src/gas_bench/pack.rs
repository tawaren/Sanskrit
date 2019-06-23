#[cfg(test)]

use super::*;


fn repeat<'a>(alloc:&'a VirtualHeapArena, obj:Object<'a>, rep:usize) -> Vec<Ptr<'a,Object<'a>>> {
    let mut builder = Vec::with_capacity(rep);
    for i in 0..rep {
        builder.push(alloc.alloc(obj).unwrap());
    }
    builder
}

#[bench]
fn bench_0(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),0)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}

#[bench]
fn bench_1(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),1)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}

#[bench]
fn bench_2(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),2)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}

#[bench]
fn bench_4(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),4)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}

#[bench]
fn bench_8(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),8)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}

#[bench]
fn bench_16(b: &mut Bencher){
    execute_code(b, |a|vec![repeat(a,Object::U8(31),16)], |_,ins|OpCode::Pack(Tag(0),ins), 1,true);
}