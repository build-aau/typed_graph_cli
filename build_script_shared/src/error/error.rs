use crate::error::{OwnedParserError, ParserError};
pub use nom::Err;
use nom::Needed;
use thiserror::Error;

use super::ComposerError;

pub type BUILDScriptResult<T> = Result<T, BUILDScriptError>;

#[derive(Error, Debug)]
pub enum BUILDScriptError {
    #[error(transparent)]
    ParserError(#[from] OwnedParserError),
    #[error(transparent)]
    ComposerError(#[from] ComposerError),
    #[error("Parser expected {0:?} more input")]
    NomIncompleteError(Needed),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl<I> From<Err<ParserError<I>>> for BUILDScriptError
where
    I: ToString,
    OwnedParserError: From<ParserError<I>>,
{
    fn from(e: Err<ParserError<I>>) -> Self {
        match e {
            Err::Error(e) 
            | Err::Failure(e) => e.into(),
            Err::Incomplete(need) => BUILDScriptError::NomIncompleteError(need),
        }
    }
}

impl<I> From<ParserError<I>> for BUILDScriptError
where
    I: ToString,
    OwnedParserError: From<ParserError<I>>,
{
    fn from(e: ParserError<I>) -> Self {
        BUILDScriptError::ParserError(e.into())
    }
}
