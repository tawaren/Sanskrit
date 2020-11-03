#[cfg(test)]
mod tests {
    pub mod op;
    pub mod struct_op;

    //Other
    pub mod void;
    pub mod pack;
    pub mod ret;
    pub mod id;

    //AdtTest
    pub mod get;
    pub mod unpack;

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

    //Frame Push/Pop Tests
    pub mod _let;
    pub mod switch;


    //Todo: Can we include this in the corresponding opcode? let, switch, ...
    //       Are they dependent on num returned values - probably they are
    //      Depends if Rollback is frame depth dependent or not -- I think not
    //       Will overcharge slightly for rollback -- as let, switch, ... acount for frame drop

    //Tests on Data
    //Todo:

    //SysCalls
    pub mod plain_hash;
    // JoinHash -- is a data call
    // EdDSA -- is a data call

    //Invoke Tests
    //Todo:





}
