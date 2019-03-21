#[cfg(test)]

use super::*;

fn build<'a>(a:&'a VirtualHeapArena, depth:usize, num_branches:u16, input:ValueRef, ret:usize) -> OpCode<'a> {
    let inner = if depth == 1 {
        SlicePtr::empty()
    } else {
        let nested = build(a,depth-1, num_branches, input, ret);
        a.copy_alloc_slice(&[nested]).unwrap()
    };
    let mut rets = a.slice_builder(ret as usize).unwrap();
    for i in 0..ret {
        rets.push(input)
    }
    let mut branches = a.slice_builder(num_branches as usize).unwrap();
    branches.push(a.alloc(Exp::Ret(inner,rets.finish())).unwrap());
    for _ in 1..num_branches {
        branches.push(a.alloc(Exp::Throw(Error::Native(NativeError::IndexError))).unwrap());
    }
    OpCode::Switch(input, branches.finish())
}

#[cfg(test)]
mod branch_1 {
    use super::*;
    const BRANCH:u16 = 1;

    #[cfg(test)]
    mod ret_0 {
        use super::*;
        const RET:usize = 0;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_1 {
        use super::*;
        const RET:usize = 1;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_2 {
        use super::*;
        const RET:usize = 2;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_4 {
        use super::*;
        const RET:usize = 4;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_8 {
        use super::*;
        const RET:usize = 8;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }
}


#[cfg(test)]
mod branch_2 {
    use super::*;
    const BRANCH:u16 = 2;

    #[cfg(test)]
    mod ret_0 {
        use super::*;
        const RET:usize = 0;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_1 {
        use super::*;
        const RET:usize = 1;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_2 {
        use super::*;
        const RET:usize = 2;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_4 {
        use super::*;
        const RET:usize = 4;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_8 {
        use super::*;
        const RET:usize = 8;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }
}


#[cfg(test)]
mod branch_4 {
    use super::*;
    const BRANCH:u16 = 4;

    #[cfg(test)]
    mod ret_0 {
        use super::*;
        const RET:usize = 0;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_1 {
        use super::*;
        const RET:usize = 1;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_2 {
        use super::*;
        const RET:usize = 2;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_4 {
        use super::*;
        const RET:usize = 4;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }

    #[cfg(test)]
    mod ret_8 {
        use super::*;
        const RET:usize = 8;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_64(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,64,BRANCH, ins[0],RET), RET,true);
        }
    }
}


#[cfg(test)]
mod branch_8 {
    use super::*;
    const BRANCH:u16 = 8;

    #[cfg(test)]
    mod ret_0 {
        use super::*;
        const RET:usize = 0;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_1 {
        use super::*;
        const RET:usize = 1;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_2 {
        use super::*;
        const RET:usize = 2;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_4 {
        use super::*;
        const RET:usize = 4;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

    }

    #[cfg(test)]
    mod ret_8 {
        use super::*;
        const RET:usize = 8;

        #[bench]
        fn bench_1(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,1,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_2(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,2,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_4(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,4,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_8(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,8,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_16(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,16,BRANCH, ins[0],RET), RET,true);
        }

        #[bench]
        fn bench_32(b: &mut Bencher){
            execute_code(b, |a|vec![vec![a.alloc(Object::Adt(0,SlicePtr::empty())).unwrap()]], |a,ins|build(a,32,BRANCH, ins[0],RET), RET,true);
        }

    }
}