use std::error;
use std::fmt;
use compiler::errors::general::*;

#[derive(Debug)]
pub enum IntegrityError {
    IncorrectModuleAssociation,
    MissingModule,
    MissingType,
    MissingFunction,
    MissingConstant,
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IntegrityError::IncorrectModuleAssociation => write!(f, "IntegrityError error: IncorrectModuleAssociation provided"),
            IntegrityError::MissingModule => write!(f, "IntegrityError error: MissingModule"),
            IntegrityError::MissingType => write!(f, "IntegrityError error: MissingType"),
            IntegrityError::MissingFunction => write!(f, "IntegrityError error: MissingFunction"),
            IntegrityError::MissingConstant => write!(f, "IntegrityError error: WMissingConstant"),
        }
    }
}

impl error::Error for IntegrityError {
    fn description(&self) -> &str {
        match *self {
            IntegrityError::IncorrectModuleAssociation => "IntegrityError error: IncorrectModuleAssociation provided",
            IntegrityError::MissingModule => "IntegrityError error: MissingModule",
            IntegrityError::MissingType => "IntegrityError error: MissingType",
            IntegrityError::MissingFunction => "IntegrityError error: MissingFunction",
            IntegrityError::MissingConstant => "IntegrityError error: WMissingConstant",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


impl From<IntegrityError> for CompilationError {
    fn from(err:IntegrityError) -> Self {
        CompilationError::IntegrityError(err)
    }
}