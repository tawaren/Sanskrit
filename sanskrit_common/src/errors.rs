use core::result::Result as OResult;

#[cfg(feature = "string_errors")]
use alloc::string::String;

//The error in case we need something to interpret (in tests)
#[cfg(not(feature = "string_errors"))]
pub type Result<T> = OResult<T,()>;

//te error if we are not interested in the details (Spares us a lot of expensive (space) string constants)
#[cfg(feature = "string_errors")]
pub type Result<T> = OResult<T,String>;

//Fast way out if we need debug infos
#[cfg(feature = "panic_errors")]
pub fn pre_error(){panic!()}

#[cfg(not(feature = "panic_errors"))]
pub fn pre_error(){}

//A Generic Error
#[cfg(not(feature = "string_errors"))]
pub fn error<T,F:FnOnce()-> &'static str>(msg:F) -> Result<T>{
    pre_error();
    Err(())
}

#[cfg(feature = "string_errors")]
pub fn error<T,F:FnOnce()-> &'static str>(msg:F) -> Result<T>{
    pre_error();
    Err(msg().into())
}