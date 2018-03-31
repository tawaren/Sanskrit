use std::error;
use std::fmt;
use compiler::errors::general::*;
use compiler::common::types::*;

#[derive(Debug)]
pub enum VerificationError {
    MissDeclaration(&'static str),
    ForwardDependency,
    UnsatisfiedPrivileges(Privileges, Privileges),
    UnsatisfiedNativePower,
    UnsatisfiedExecutionMode(ExecutionMode,ExecutionMode),
    UnsatisfiedDeclaration(&'static str),
    ForbiddenModifier(&'static str),
    BodyInclusion(&'static str),
    ImportedTypeMismatch
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VerificationError::MissDeclaration(ref what) => write!(f, "VerificationError error: Declared {} did not match", what),
            VerificationError::ForwardDependency => write!(f, "VerificationError error: Imports can only depend on previous imports"),
            VerificationError::UnsatisfiedNativePower => write!(f, "VerificationError error: Native types must be Open"),
            VerificationError::UnsatisfiedPrivileges(declared, needed) => write!(f, "VerificationError error: {:?} does not satisfy {:?}", declared, needed),
            VerificationError::UnsatisfiedExecutionMode(declared, needed) => write!(f, "VerificationError error: {:?} does not satisfy {:?}", declared, needed),
            VerificationError::BodyInclusion(ref what) => write!(f, "VerificationError error: {} ", what),
            VerificationError::UnsatisfiedDeclaration(ref what) => write!(f, "VerificationError error: Wrong structure for {} ", what),
            VerificationError::ForbiddenModifier(ref what) => write!(f, "VerificationError error: Modifier {} not allowed here ", what),
            VerificationError::ImportedTypeMismatch => write!(f, "VerificationError error: Imported type unequal to specified type "),
        }
    }
}

impl error::Error for VerificationError {
    fn description(&self) -> &str {
        match *self {
            VerificationError::MissDeclaration(..) => "VerificationError error: MissDeclaration",
            VerificationError::ForwardDependency => "VerificationError error: ForwardDependency",
            VerificationError::UnsatisfiedPrivileges(..) => "VerificationError error: UnsatisfiedBehaviour",
            VerificationError::UnsatisfiedNativePower => "VerificationError error: UnsatisfiedNativeBehaviour",
            VerificationError::UnsatisfiedExecutionMode(..) => "VerificationError error: UnsatisfiedExecutionMode",
            VerificationError::BodyInclusion(..) => "VerificationError error: BodyInclusion",
            VerificationError::UnsatisfiedDeclaration(..) => "VerificationError error: UnsatisfiedDeclaration",
            VerificationError::ForbiddenModifier(..) => "VerificationError error: ForbiddenModifier",
            VerificationError::ImportedTypeMismatch => "VerificationError error: ImportedTypeMismatch",

        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


impl From<VerificationError> for CompilationError {
    fn from(err:VerificationError) -> Self {
        CompilationError::VerificationError(err)
    }
}