use std::fmt::Write;
use std::fs::write;

use std::path::Path;

use crate::error::{ComposerError, ComposerResult, ParserError, ParserErrorKind, ParserResult};
use crate::{BUILDScriptError, BUILDScriptResult, InputType};

use super::{append_parser_error, Mark};

pub trait ParserDeserialize<I: InputType>
where
    Self: Sized,
{
    /// Parse a single item as part of a long chain of parsers
    fn parse(s: I) -> ParserResult<I, Self>;

    /// Parse a single item as the starting point for a parser
    ///
    /// This will mark the start of the input so errors are able to retrieve the entire input stream
    fn deserialize<S: Into<I>>(s: S) -> BUILDScriptResult<Self>
    where
        BUILDScriptError: From<nom::Err<ParserError<I>>>,
    {
        let value = append_parser_error(Self::parse, ParserErrorKind::EndOfFile)(s.into())
            .map(|(_, value)| value)?;
        Ok(value)
    }
}

pub trait ParserDeserializeTo<I: InputType, O> {
    fn deserialize<S: Into<I>>(self, s: S) -> BUILDScriptResult<O>
    where
        BUILDScriptError: From<nom::Err<ParserError<I>>>;
}

impl<I, O, F> ParserDeserializeTo<I, O> for F
where
    I: InputType,
    F: Fn(I) -> ParserResult<I, O> + Sized,
{
    fn deserialize<S: Into<I>>(self, s: S) -> BUILDScriptResult<O>
    where
        BUILDScriptError: From<nom::Err<ParserError<I>>>,
    {
        let value = append_parser_error(self, ParserErrorKind::EndOfFile)(s.into())
            .map(|(_, value)| value)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Default)]
pub struct ComposeContext {
    pub indents: usize,
}

impl ComposeContext {
    pub fn increment_indents(mut self, indents: usize) -> Self {
        self.indents += indents;
        self
    }

    pub fn set_indents(mut self, indents: usize) -> Self {
        self.indents = indents;
        self
    }

    pub fn create_indents(&self) -> String {
        (0..self.indents).map(|_| "    ").collect()
    }
}

pub trait ParserSerialize
where
    Self: Sized,
{
    /// Write the content of a single item
    fn compose<W: Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()>;

    /// Write the content of the item to a string
    fn serialize_to_string(&self) -> BUILDScriptResult<String> {
        let mut s = String::new();
        self.compose(&mut s, Default::default())?;
        Ok(s)
    }

    /// Write the content of the item to a file
    fn serialize_to_file<P: AsRef<Path>>(&self, p: P) -> BUILDScriptResult<()> {
        let mut s = String::new();
        self.compose(&mut s, Default::default())?;

        write(p, s).map_err(|e| ComposerError::from(e))?;

        Ok(())
    }
}

pub trait Marked<I> {
    /// Retrivve the marker telling where in the input file the item originates from
    fn marker(&self) -> &Mark<I>;
}
