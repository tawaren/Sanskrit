#[cfg(test)]

use super::*;

const OP:[Operand;1] = [Operand::Derive];

#[bench]
fn bench(b: &mut Bencher) {
    execute_native(b, |a|vec![vec![a.alloc(Object::Data(a.copy_alloc_slice(&[1; 20]).unwrap())).unwrap(), a.alloc(Object::Data(a.copy_alloc_slice(&[31; 20]).unwrap())).unwrap()]], &OP, true);
}