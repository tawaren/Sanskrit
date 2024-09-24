
use std::borrow::BorrowMut;
use std::cell::{RefCell, Cell};
use std::fmt;
use sanskrit_common::store::{StorageClass, Store};
use sanskrit_common::model::{HASH_SIZE, hash_from_slice, Hash};
use fluid_let::fluid_let;
use sanskrit_sled_store::SledStore;
use sanskrit_common::errors::{error, error_to_string, Result as SResult};


#[derive(Debug, Clone, Copy)]
pub struct ExitCode(u32);

pub fn gen_code(res:&ExitCode) -> &str {
    match res {
        ExitCode(1) => "abort due to error",
        ExitCode(2) => "Memory access failed",
        ExitCode(3) => "Out of gas",
        ExitCode(4) => "Load failed",
        _ => "Unknown Exit Code"
    }

}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", gen_code(self))
    }
}

impl std::error::Error for ExitCode {}

fluid_let!(static INPUT: Vec<u8>);
fluid_let!(static STORAGE: SledStore);


pub struct CompilerInstance{ }

impl CompilerInstance {
    pub fn with_compiler_result<R>(f: impl FnOnce(&mut Self) -> SResult<R>) -> SResult<R> {
        let mut inst = CompilerInstance {};
        f(&mut inst)
    }
}

