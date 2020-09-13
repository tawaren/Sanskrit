use core::cell::Cell;

pub struct Limiter {
    pub max_functions:usize,
    pub max_nesting:usize,
    pub max_used_nesting:Cell<usize>,
    pub produced_functions:Cell<usize>

}