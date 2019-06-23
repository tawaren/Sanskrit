#[cfg(test)]

use super::*;

const OP:[Operand;1] = [Operand::Concat;1];

#[cfg(test)]
mod data1 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 1]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}


#[cfg(test)]
mod data10 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 10]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}


#[cfg(test)]
mod data20 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}

#[cfg(test)]
mod data50 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 50]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}

#[cfg(test)]
mod data100 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 100]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}


#[cfg(test)]
mod data200 {

    use super::*;

    #[bench]
    fn bench_data1(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data10(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data20(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data50(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data100(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()]], &OP, true);
    }

    #[bench]
    fn bench_data200(b: &mut Bencher) {
        execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]], &OP, true);
    }


    #[bench]
    fn bench_all(b: &mut Bencher){
        execute_native(b, |a|{
            vec![
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 1]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 10]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 50]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 100]).unwrap())).unwrap()],
                vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 200]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 200]).unwrap())).unwrap()]
            ]
        }, &OP, true);
    }
}
