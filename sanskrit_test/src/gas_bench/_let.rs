#[cfg(test)]

use super::*;

fn build<'a>(a:&'a VirtualHeapArena, depth:usize, ret:u16) -> OpCode<'a> {
    let inner = if depth == 1 {
        SlicePtr::empty()
    } else {
        let nested = build(a,depth-1,ret);
        a.copy_alloc_slice(&[nested]).unwrap()
    };
    let mut rets = a.slice_builder(ret as usize).unwrap();
    for i in 0..ret {
        rets.push(ValueRef(i))
    }
    OpCode::Let(a.alloc(Exp::Ret(inner,rets.finish())).unwrap())
}

fn build_throw<'a>(a:&'a VirtualHeapArena, depth:usize) -> OpCode<'a> {
    fn build_throw_inner<'a>(a:&'a VirtualHeapArena, depth:usize) -> OpCode<'a> {
        if depth == 1 {
            OpCode::Let(a.alloc(Exp::Throw(Error::Native(NativeError::IndexError))).unwrap())
        } else {
            let nested = build_throw(a,depth-1);
            let inner = a.copy_alloc_slice(&[nested]).unwrap();
            OpCode::Let(a.alloc(Exp::Ret(inner,SlicePtr::empty())).unwrap())
        }
    }

    let err = a.copy_alloc_slice(&[(Error::Native(NativeError::IndexError),a.alloc(Exp::Ret(SlicePtr::empty(),SlicePtr::empty())).unwrap())]).unwrap();
    
    if depth == 1 {
        OpCode::Try(a.alloc(Exp::Throw(Error::Native(NativeError::IndexError))).unwrap(),err)
    } else {
        let nested = build_throw_inner(a,depth-1);
        let inner = a.copy_alloc_slice(&[nested]).unwrap();
        OpCode::Try(a.alloc(Exp::Ret(inner,SlicePtr::empty())).unwrap(),err)
    }
}

fn input<'a>(a:&'a VirtualHeapArena, obj:Object<'a>, num:usize) -> Vec<Ptr<'a, Object<'a>>> {
    let mut ret = Vec::with_capacity(num);
    for _ in 0..num {
        ret.push(a.alloc(obj).unwrap())
    }
    ret
}

#[cfg(test)]
mod ret_0 {
    use super::*;
    const RET:usize = 0;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,1,RET as u16), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,2,RET as u16), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,4,RET as u16), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,8,RET as u16), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,16,RET as u16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,32,RET as u16), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,64,RET as u16), RET,true);
    }

}


#[cfg(test)]
mod ret_1 {
    use super::*;
    const RET:usize = 1;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,1,RET as u16), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,2,RET as u16), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,4,RET as u16), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,8,RET as u16), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,16,RET as u16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,32,RET as u16), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,64,RET as u16), RET,true);
    }

}

#[cfg(test)]
mod ret_2 {
    use super::*;
    const RET:usize = 2;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,1,RET as u16), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,2,RET as u16), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,4,RET as u16), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,8,RET as u16), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,16,RET as u16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,32,RET as u16), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,64,RET as u16), RET,true);
    }

}

#[cfg(test)]
mod ret_4 {
    use super::*;
    const RET:usize = 4;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,1,RET as u16), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,2,RET as u16), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,4,RET as u16), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,8,RET as u16), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,16,RET as u16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,32,RET as u16), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,64,RET as u16), RET,true);
    }

}

#[cfg(test)]
mod ret_8 {
    use super::*;
    const RET:usize = 8;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,1,RET as u16), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,2,RET as u16), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,4,RET as u16), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,8,RET as u16), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,16,RET as u16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,32,RET as u16), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build(a,64,RET as u16), RET,true);
    }

}

#[cfg(test)]
mod ret_throw {
    use super::*;
    const RET:usize = 0;

    #[bench]
    fn bench_1(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,1), RET,true);
    }

    #[bench]
    fn bench_2(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,2), RET,true);
    }

    #[bench]
    fn bench_4(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,4), RET,true);
    }

    #[bench]
    fn bench_8(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,8), RET,true);
    }

    #[bench]
    fn bench_16(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,16), RET,true);
    }

    #[bench]
    fn bench_32(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,32), RET,true);
    }

    #[bench]
    fn bench_64(b: &mut Bencher){
        execute_code(b, |a|vec![input(a,Object::U8(32),RET)], |a,_|build_throw(a,64), RET,true);
    }

}