use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::*;
use nom::branch::alt;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Clone, Hash, Dummy, Serialize, Deserialize)]
#[serde(bound = "I: Default")]
pub struct Quantifier<I> {
    // The upper bounds are inclusive
    pub bounds: Option<(LowerBound, u32)>,
    #[serde(skip)]
    marker: Mark<I>,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Dummy, Serialize, Deserialize,
)]
pub enum LowerBound {
    Zero,
    One,
}

impl<I> Quantifier<I> {
    pub fn new(bounds: Option<(LowerBound, u32)>, marker: Mark<I>) -> Self {
        Quantifier { bounds, marker }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Quantifier<O>
    where
        F: FnMut(I) -> O,
    {
        Quantifier {
            bounds: self.bounds,
            marker: self.marker.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Quantifier<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        // match [ 0 ..= {u32}]
        let (s, (res, marker)) = context(
            "Parsing Quantifier",
            marked(opt(surrounded(
                '[',
                ws(tuple((
                    context(
                        "Parsing lower bounds",
                        alt((
                            map(char('0'), |_| LowerBound::Zero),
                            map(char('1'), |_| LowerBound::One),
                        )),
                    ),
                    ws(pair(char('.'), char('.'))),
                    context("Parsing upper bounds", parsers::u32),
                ))),
                ']',
            ))),
        )(s)?;

        let bounds = res.map(|(lower, _, upper)| (lower, upper));

        Ok((s, Quantifier { bounds, marker }))
    }
}

impl<I> ParserSerialize for Quantifier<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        let indents = ctx.create_indents();
        match &self.bounds {
            Some((lower, upper)) => match lower {
                LowerBound::Zero => write!(f, "{indents}[0..{upper}]")?,
                LowerBound::One => write!(f, "{indents}[1..{upper}]")?,
            },
            None => (),
        }
        Ok(())
    }
}

impl<I: Default> Default for Quantifier<I> {
    fn default() -> Self {
        Quantifier::new(None, Mark::null())
    }
}

impl<I> Display for Quantifier<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.bounds {
            Some((lower, upper)) => match lower {
                LowerBound::Zero => write!(f, "[0..{upper}]")?,
                LowerBound::One => write!(f, "[0..{upper}]")?,
            },
            None => (),
        }
        Ok(())
    }
}

impl<I> PartialEq for Quantifier<I> {
    fn eq(&self, other: &Self) -> bool {
        self.bounds.eq(&other.bounds)
    }
}

impl<I> Eq for Quantifier<I> {}

impl<I> PartialOrd for Quantifier<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I> Ord for Quantifier<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bounds.cmp(&other.bounds)
    }
}

impl<I> Marked<I> for Quantifier<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

compose_test! {quantity_compose, Quantifier<I>}
