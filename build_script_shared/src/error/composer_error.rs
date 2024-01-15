pub use nom::Err;
use thiserror::Error;

pub type ComposerResult<T> = Result<T, ComposerError>;

#[derive(Error, Debug)]
pub enum ComposerError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),
}