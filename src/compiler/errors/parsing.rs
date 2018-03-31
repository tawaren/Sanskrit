use std::error;
use std::io;
use std::fmt;
use compiler::errors::general::*;

#[derive(Debug)]
pub enum ParsingError {
    WrongHashSize{expected:usize, actual:usize},
    ViewIndexAccessError{requested:usize, len:usize},
    WrongMagicNumber{expected:u8,actual:u8},
    WrongInputVersion{expected:u8,actual:u8},
    WrongPrivilegesEncoding{provided:u16,max:u16},
    WrongEnumEncoding{provided:u8,max:u8,enum_name:&'static str},
    MissingBody,
    IOError(io::Error),
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParsingError::WrongPrivilegesEncoding{ref provided, ref max} => write!(f, "ParsingError error: WrongPrivilegesEncoding provided: {} max: {}", provided, max),
            ParsingError::WrongEnumEncoding{ref provided, ref max, ref enum_name} => write!(f, "ParsingError error: WrongEnumEncoding provided: {} max: {} for: {}", provided, max, enum_name),
            ParsingError::WrongMagicNumber{ref expected, ref actual} => write!(f, "ParsingError error: WrongMagicNumber expected: {} actual: {}", expected, actual),
            ParsingError::WrongInputVersion{ref expected, ref actual} => write!(f, "ParsingError error: WrongInputVersion expected: {} actual: {}", expected, actual),
            ParsingError::WrongHashSize{ref expected, ref actual} => write!(f, "ParsingError error: WrongHashSize expected: {} actual: {}", expected, actual),
            ParsingError::ViewIndexAccessError{ref requested, ref len} => write!(f, "ParsingError error: ViewIndexAccessError requested: {} len: {}", requested, len),
            ParsingError::IOError(ref err) => fmt::Display::fmt(&err,f),
            ParsingError::MissingBody => write!(f, "ParsingError error: Provided input has no body"),
        }
    }
}

impl error::Error for ParsingError {
    fn description(&self) -> &str {
        match *self {
            ParsingError::WrongPrivilegesEncoding {..} => "ParsingError error: WrongPrivilegesEncoding",
            ParsingError::WrongEnumEncoding{..} => "ParsingError error: WrongEnumEncoding",
            ParsingError::WrongMagicNumber{..} => "ParsingError error: WrongMagicNumber",
            ParsingError::WrongInputVersion{..} => "ParsingError error: WrongInputVersion",
            ParsingError::WrongHashSize{..} => "ParsingError error: WrongHashSize",
            ParsingError::ViewIndexAccessError{..} => "ParsingError error: ViewIndexAccessError",
            ParsingError::IOError(ref err) => err.description(),
            ParsingError::MissingBody => "ParsingError error: MissingBody",

        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ParsingError::IOError(ref err) => Some(err),
            _ => None
        }

    }
}


impl From<ParsingError> for CompilationError {
    fn from(err:ParsingError) -> Self {
        CompilationError::ParsingError(err)
    }
}

impl From<io::Error> for ParsingError {
    fn from(err:io::Error) -> Self {
        ParsingError::IOError(err)
    }
}