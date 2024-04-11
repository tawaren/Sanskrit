use core::result::Result as OResult;

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
pub fn error<T,F:FnOnce()-> &'static str>(_msg:F) -> Result<T>{
    pre_error();
    Err(())
}

#[cfg(not(feature = "string_errors"))]
pub fn owned_error<T,F:FnOnce()-> String>(_msg:F) -> Result<T>{
    pre_error();
    Err(())
}

#[cfg(feature = "string_errors")]
pub fn error<'a, T,F:FnOnce()-> &'a str>(msg:F) -> Result<T>{
    pre_error();
    Err(msg().to_owned())
}

#[cfg(feature = "string_errors")]
pub fn owned_error<'a, T,F:FnOnce()-> String>(msg:F) -> Result<T>{
    pre_error();
    Err(msg())
}

#[cfg(not(feature = "string_errors"))]
pub fn error_to_string(_err:&()) -> &str {
    "error was not captured"
}

#[cfg(feature = "string_errors")]
pub fn error_to_string(err:&str) -> &str {
    err
}
