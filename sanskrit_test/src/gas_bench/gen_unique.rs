#[cfg(test)]

use super::*;
const OP:[Operand;1] = [Operand::GenUnique;1];

#[bench]
fn bench(b: &mut Bencher) {
    execute_multi_ret_native(b, |a|vec![vec![a.alloc(Object::Context(0)).unwrap()]], &OP, 2, true);
}
