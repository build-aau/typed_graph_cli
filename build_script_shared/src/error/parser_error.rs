use std::fmt::Display;
use std::ops::Deref;

use nom::error::{ContextError, ErrorKind, ParseError};
use nom::{Err, IResult};

use crate::parsers::Marked;
use crate::InputType;

pub type ParserResult<I, T> = IResult<I, T, ParserError<I>>;
pub type ParserSlimResult<I, T> = Result<T, Err<ParserError<I>>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParserError<I> {
    pub errors: Vec<(I, ParserErrorKind)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParserErrorKind {
    // Marks the end of the file so all errors contain the entirety of the of the input
    EndOfFile,
    FailedToParseInteger,
    CyclicReference,
    DuplicateDefinition(String),
    FirstOccurance,
    MissingRequiredField(String),
    ChangedProtectedField(String),
    InvalidAttribute(String),
    UnexpectedFieldType(String, String),
    Context(&'static str),
    OwnedContext(String),
    ErrorKind(ErrorKind),
    ExpectedChar(char, Option<char>),
    UnknownReference(String),
    UnexpectedGenericCount(String, usize, usize),
    InvalidTypeConvertion(String, String),
    UnusedGeneric,
}

impl<I> ParserError<I> {
    pub fn new(marker: I, e: ParserErrorKind) -> ParserError<I> {
        ParserError {
            errors: vec![(marker, e)],
        }
    }

    pub fn new_at<Marker>(ident: &Marker, e: ParserErrorKind) -> ParserError<I>
    where
        Marker: Marked<I>,
        I: Clone,
    {
        ParserError {
            errors: vec![(ident.marker().deref().clone(), e)],
        }
    }

    pub fn push(&mut self, input: I, e: ParserErrorKind) {
        self.errors.push((input, e));
    }

    pub fn new_single(whole_input: I, marker: I, e: ParserErrorKind) -> ParserError<I> {
        ParserError {
            errors: vec![(whole_input, ParserErrorKind::EndOfFile), (marker, e)],
        }
    }
}

// E: ParseError<I> + ContextError<I>

impl<I: InputType> ParseError<I> for ParserError<I> {
    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, ParserErrorKind::ErrorKind(kind)));
        other
    }

    fn from_char(input: I, char: char) -> Self {
        let actual = input.iter_elements().next();
        ParserError {
            errors: vec![(input, ParserErrorKind::ExpectedChar(char, actual))],
        }
    }

    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ParserError {
            errors: vec![(input, ParserErrorKind::ErrorKind(kind))],
        }
    }
}

impl<I> ContextError<I> for ParserError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, ParserErrorKind::Context(ctx)));
        other
    }
}

impl<I: Clone, M: Marked<I>> FromIterator<(M, ParserErrorKind)> for ParserError<I> {
    fn from_iter<T: IntoIterator<Item = (M, ParserErrorKind)>>(iter: T) -> Self {
        Self {
            errors: iter
                .into_iter()
                .map(|(m, e)| (m.marker().deref().clone(), e))
                .collect(),
        }
    }
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorKind::ExpectedChar(c, actual) => {
                if let Some(actual) = actual {
                    write!(f, "Expected '{c}', Found {actual}")?;
                } else {
                    write!(f, "Expected '{c}', Got end of input")?;
                }
            }
            ParserErrorKind::FailedToParseInteger => {
                write!(f, "Expected integer")?;
            }
            ParserErrorKind::MissingRequiredField(field_name) => {
                write!(f, "Missing required field {field_name}")?;
            }
            ParserErrorKind::ChangedProtectedField(field_name) => {
                write!(f, "Cannot make changes to protected field {field_name}")?;
            }
            ParserErrorKind::CyclicReference => {
                write!(f, "Detected cyclic reference to type")?;
            }
            ParserErrorKind::FirstOccurance => {
                write!(f, "First occurrence")?;
            }
            ParserErrorKind::DuplicateDefinition(name) => {
                write!(f, "Multiple definitions of {name:?}")?;
            }
            ParserErrorKind::InvalidAttribute(allowed) => {
                write!(f, "Invalid attribute allowed attributes are {allowed:?}")?;
            }
            ParserErrorKind::ErrorKind(e) => {
                write!(f, "Encountered error {e:?}")?;
            }
            ParserErrorKind::UnexpectedFieldType(field, ty) => {
                write!(f, "Unexpected {field} type {ty}")?;
            }
            ParserErrorKind::Context(ctx) => {
                write!(f, "{ctx}")?;
            }
            ParserErrorKind::OwnedContext(ctx) => {
                write!(f, "{ctx}")?;
            }
            ParserErrorKind::UnknownReference(field_type) => {
                write!(f, "Unknown reference {field_type}\n")?
            }
            ParserErrorKind::UnexpectedGenericCount(field_type, expected, actual) => write!(
                f,
                "{field_type} takes {expected} generic argument(s) but {actual} was provided"
            )?,
            ParserErrorKind::UnusedGeneric => {
                write!(f, "Generic is never used")?;
            }
            ParserErrorKind::InvalidTypeConvertion(old, new) => {
                write!(f, "Invalid type convertion from {old} to {new}")?;
            }
            ParserErrorKind::EndOfFile => {}
        }

        Ok(())
    }
}
