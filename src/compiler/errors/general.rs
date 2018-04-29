use std::error;
use std::option;
use std::fmt;
use compiler::errors::parsing::*;
use compiler::errors::integrity::*;
use compiler::errors::verification::*;

#[derive(Debug)]
pub enum CompilationError {
    ParsingError(ParsingError),
    IntegrityError(IntegrityError),
    VerificationError(VerificationError),
    NoneError

}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CompilationError::ParsingError(ref err) => fmt::Display::fmt(&err,f),
            CompilationError::IntegrityError(ref err) =>  fmt::Display::fmt(&err,f),
            CompilationError::VerificationError(ref err) =>  fmt::Display::fmt(&err,f),
            CompilationError::NoneError => write!(f, "NoneError"),
        }
    }
}

impl error::Error for CompilationError {
    fn description(&self) -> &str {
        match *self {
            CompilationError::ParsingError(ref err) => err.description(),
            CompilationError::IntegrityError(ref err) => err.description(),
            CompilationError::VerificationError(ref err) => err.description(),
            CompilationError::NoneError => "NoneError",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            CompilationError::ParsingError(ref err) => Some(err),
            CompilationError::IntegrityError(ref err) => Some(err),
            CompilationError::VerificationError(ref err) => Some(err),
            CompilationError::NoneError => None,
        }
    }
}

impl From<option::NoneError> for CompilationError {
    fn from(_:option::NoneError) -> Self{
        panic!();
            CompilationError::NoneError
    }
}