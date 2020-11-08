pub mod tests {
    pub mod op;
    pub mod call_op;
    pub mod struct_op;

    //Other
    pub mod void;
    pub mod pack;
    pub mod ret;
    pub mod id;
    pub mod _let;
    pub mod try_succ;
    pub mod try_fail;

    //AdtTest
    pub mod get;
    pub mod unpack;
    pub mod switch;

    //Numeric Tests
    pub mod add;
    pub mod sub;
    pub mod mul;
    pub mod div;
    pub mod and;
    pub mod or;
    pub mod xor;
    pub mod not;

    //Compare Tests
    pub mod eq;
    pub mod gt;
    pub mod gte;
    pub mod lt;
    pub mod lte;


    //Todo: Can we include this in the corresponding opcode? let, switch, ...
    //       Are they dependent on num returned values - probably they are
    //      Depends if Rollback is frame depth dependent or not -- I think not
    //       Will overcharge slightly for rollback -- as let, switch, ... acount for frame drop

    //Tests on Data
    //Todo:

    //SysCalls
    pub mod plain_hash;
    pub mod join_hash;

    // JoinHash -- is a data call
    // EdDSA -- is a data call

    //Invoke Tests
    pub mod call;





}
