use thiserror::Error;

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[derive(Error, Debug)]
pub enum ChangeSetError {
    #[error("Incompatible schema version, expected {expected:#16x} recieved {recieved:#16x} when updating {old_version} to {new_version}")]
    IncompatibleSchemaVersion {
        expected: u64,
        recieved: u64,
        old_version: String,
        new_version: String
    },
    #[error("Failed to apply changeset {old_version} => {new_version} as the produced hash {recieved:#16x} does not correspond to the provided one {expected:#16x}")]
    UpdateFailed {
        expected: u64,
        recieved: u64,
        old_version: String,
        new_version: String
    },
    #[error("Invalid comparison between types {type0} and {type1}")]
    InvalidTypeComparison {
        type0: String,
        type1: String,
    },
    #[error("Changes to {0} is currently not supported")]
    UnsupportedChange(String),
    #[error("Attempted to {action} but failed with {reason}")]
    InvalidAction {
        action: String,
        reason: String
    },
    #[error("Invalid field path {path} to {target}")]
    InvalidFieldPath {
        path: String,
        target: String,
    },
    #[error("Expected to recieve field path")]
    MissingFieldPath,

}