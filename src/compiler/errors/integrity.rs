use std::error;
use std::fmt;
use compiler::errors::general::*;

#[derive(Debug)]
pub enum IntegrityError {
    IncorrectModuleAssociation,
    MissingModule,
    ModuleAlreadyExists,
    MissingType,
    TypeAlreadyExists,
    MissingFunction,
    FunctionAlreadyExists,
    MissingConstant,
    ConstantAlreadyExists
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IntegrityError::IncorrectModuleAssociation => write!(f, "IntegrityError error: IncorrectModuleAssociation provided"),
            IntegrityError::MissingModule => write!(f, "IntegrityError error: MissingModule"),
            IntegrityError::MissingType => write!(f, "IntegrityError error: MissingType"),
            IntegrityError::MissingFunction => write!(f, "IntegrityError error: MissingFunction"),
            IntegrityError::MissingConstant => write!(f, "IntegrityError error: MissingConstant"),
            IntegrityError::ModuleAlreadyExists => write!(f, "IntegrityError error: ModuleAlreadyExists"),
            IntegrityError::TypeAlreadyExists => write!(f, "IntegrityError error: TypeAlreadyExists"),
            IntegrityError::FunctionAlreadyExists => write!(f, "IntegrityError error: FunctionAlreadyExists"),
            IntegrityError::ConstantAlreadyExists => write!(f, "IntegrityError error: ConstantAlreadyExists"),
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
            IntegrityError::MissingConstant => "IntegrityError error: MissingConstant",
            IntegrityError::ModuleAlreadyExists => "IntegrityError error: ModuleAlreadyExists",
            IntegrityError::TypeAlreadyExists => "IntegrityError error: TypeAlreadyExists",
            IntegrityError::FunctionAlreadyExists => "IntegrityError error: FunctionAlreadyExists",
            IntegrityError::ConstantAlreadyExists => "IntegrityError error: ConstantAlreadyExists",

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