
//Note we rounded up - No freactions
// Based on sanskrit_bench - main
//  Current Run on home laptop with approx 3/5 as factor (in release)
pub mod gas {

    //16
    pub fn call(args:usize) -> u64 {
        20 + 2 * (args as u64)
    }

    //todo: may be cheaper as arguments do not need reajusting after first run
    //todo: may be more expensive as we need the abort check
    //todo: measure
    pub fn repeated_call(args:usize, reps:u64) -> u64 {
        reps*call(args)
    }

    pub fn void() -> u64 {
        3
    }

    pub fn gas() -> u64 { 2 /*Note this one is purely guessed*/ }

    pub fn sig(fields:usize) -> u64 {
       pack(fields)
    }

    //Remeasuring gave
    pub fn _let() -> u64 { 20 }

    pub fn unpack(fields:usize) -> u64 {
        3 + 2*(fields as u64)
    }

    pub fn field() -> u64 {
        5
    }

    pub fn pack(fields:usize) -> u64 {
        13 + 2*(fields as u64)
    }

    pub fn switch() -> u64 {
        20
    }

    //This looks strange why so expensive (+4?)
    pub fn ret(rets:usize) -> u64 {
        5 + 2*(rets as u64)
    }

    pub fn rollback() -> u64 {
        5
    }

    pub fn try() -> u64 { 20 }
}
