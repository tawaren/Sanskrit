
pub mod gas {

    //todo: maybe external call as different??? as it may have overhead??
    pub fn call(args:usize) -> u64 {
        //1.4 was rounded to 1.5
        14 + (3*(args as u64))/2//70 + 7*args
    }

    pub fn void() -> u64 {
        3 /*Note this one is purely guessed*/
    }


    pub fn sig(fields:usize) -> u64 {
        13 + (fields as u64)
    }

    pub fn _let() -> u64 { 70 }

    /*
    pub fn lit(lit_size:u16) -> u64 {
        (13 + lit_size/50) as u64
    }
    */
    pub fn unpack(fields:usize) -> u64 {
        3 + (fields as u64)/2
    }

    pub fn field() -> u64 {
        4
    }

    pub fn pack(fields:usize) -> u64 {
        13 + (fields as u64)
    }

    pub fn switch() -> u64 {
        16
    }

    pub fn ret(rets:usize) -> u64 {
        5 + 5*(rets as u64)
    }

    pub fn throw() -> u64 {
        5
    }

}
