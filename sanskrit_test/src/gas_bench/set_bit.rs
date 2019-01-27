#[cfg(test)]

use super::*;

const OP:[Operand;1] = [Operand::SetBit;1];

#[bench]
fn bench_data1(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap(), a.alloc(Object::U16((1*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data10(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap(), a.alloc(Object::U16((10*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data20(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap(), a.alloc(Object::U16((20*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data50(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap(), a.alloc(Object::U16((50*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data100(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap(), a.alloc(Object::U16((100*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_data200(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap(), a.alloc(Object::U16((200*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]], &OP, true);
}

#[bench]
fn bench_all(b: &mut Bencher){
    execute_native(b, |a|{
        vec![
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap(), a.alloc(Object::U16((1*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap(), a.alloc(Object::U16((10*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap(), a.alloc(Object::U16((20*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap(), a.alloc(Object::U16((50*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap(), a.alloc(Object::U16((100*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()],
            vec![a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap(), a.alloc(Object::U16((200*8)-1)).unwrap(), a.alloc(Object::Adt(1,SlicePtr::empty())).unwrap()]
        ]
    }, &OP, true);
}