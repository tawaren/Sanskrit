use core::result::Result as OResult;

#[cfg(feature = "string_errors")]
use alloc::string::String;
#[cfg(feature = "string_errors")]
use alloc::borrow::ToOwned;

//The error in case we need something to interpret (in tests)
#[cfg(not(feature = "string_errors"))]
pub type ErrorType = ();

//te error if we are not interested in the details (Spares us a lot of expensive (space) string constants)
#[cfg(feature = "string_errors")]
pub type ErrorType = String;

pub type Result<T> = OResult<T,ErrorType>;

//Fast way out if we need debug infos
#[cfg(feature = "panic_errors")]
pub fn pre_error(){panic!()}

#[cfg(not(feature = "panic_errors"))]
pub fn pre_error(){}

//A Generic Error
#[cfg(not(feature = "string_errors"))]
pub fn error<T,F:FnOnce()-> &str>(msg:F) -> Result<T>{
    pre_error();
    Err(())
}

#[cfg(feature = "string_errors")]
pub fn error<'a, T,F:FnOnce()-> &'a str>(msg:F) -> Result<T>{
    pre_error();
    Err(msg().to_owned())
}

#[cfg(not(feature = "string_errors"))]
pub fn error_to_string(err:&()) -> &str {
    "error was not captured"
}

#[cfg(feature = "string_errors")]
pub fn error_to_string(err:&str) -> &str {
    err
}