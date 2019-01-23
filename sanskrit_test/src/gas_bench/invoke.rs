#[cfg(test)]

use super::*;

fn repeat<'a>(alloc:&'a VirtualHeapArena, obj:Object<'a>, rep:usize) -> Vec<Ptr<'a,Object<'a>>> {
    let mut builder = Vec::with_capacity(rep);
    for i in 0..rep {
        builder.push(alloc.alloc(obj).unwrap());
    }
    builder
}

fn dummy_fun<'a>(alloc:&'a VirtualHeapArena, ret:usize) -> Ptr<'a,Exp<'a>> {
    let mut ret_vals = alloc.slice_builder(ret).unwrap();
    for i in 0..ret {
        ret_vals.push(ValueRef(i as u16))
    }
    alloc.alloc(Exp::Ret(SlicePtr::empty(), ret_vals.finish() )).unwrap()
}

#[cfg(test)]
mod param_0{
    use super::*;

    const PARAM:usize = 0;

    #[bench]
    fn bench_ret_0(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 0, |a|vec![dummy_fun(a,0)],true);
    }
}

#[cfg(test)]
mod param_1{
    use super::*;

    const PARAM:usize = 1;

    #[bench]
    fn bench_ret_0(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 0, |a|vec![dummy_fun(a,0)],true);
    }

    #[bench]
    fn bench_ret_1(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 1, |a|vec![dummy_fun(a,1)],true);
    }

}

#[cfg(test)]
mod param_2{
    use super::*;

    const PARAM:usize = 2;

    #[bench]
    fn bench_ret_0(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 0, |a|vec![dummy_fun(a,0)],true);
    }

    #[bench]
    fn bench_ret_1(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 1, |a|vec![dummy_fun(a,1)],true);
    }

    #[bench]
    fn bench_ret_2(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 2, |a|vec![dummy_fun(a,2)],true);
    }
}

#[cfg(test)]
mod param_4{
    use super::*;

    const PARAM:usize = 4;

    #[bench]
    fn bench_ret_0(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 0, |a|vec![dummy_fun(a,0)],true);
    }

    #[bench]
    fn bench_ret_1(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 1, |a|vec![dummy_fun(a,1)],true);
    }

    #[bench]
    fn bench_ret_2(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 2, |a|vec![dummy_fun(a,2)],true);
    }

    #[bench]
    fn bench_ret_4(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 4, |a|vec![dummy_fun(a,4)],true);
    }

}

#[cfg(test)]
mod param_8{
    use super::*;

    const PARAM:usize = 8;

    #[bench]
    fn bench_ret_0(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 0, |a|vec![dummy_fun(a,0)],true);
    }

    #[bench]
    fn bench_ret_1(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 1, |a|vec![dummy_fun(a,1)],true);
    }

    #[bench]
    fn bench_ret_2(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 2, |a|vec![dummy_fun(a,2)],true);
    }

    #[bench]
    fn bench_ret_4(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 4, |a|vec![dummy_fun(a,4)],true);
    }

    #[bench]
    fn bench_ret_8(b: &mut Bencher){
        execute_code_with_extra_fun(b, |a|vec![repeat(a,Object::U8(31),PARAM)], |_,ins|OpCode::Invoke(1,ins), 8, |a|vec![dummy_fun(a,8)],true);
    }

}



