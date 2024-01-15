use std::fmt::Display;

use fake::Dummy;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use build_script_shared::{InputType, compose_test};
use build_script_shared::error::{ParserResult, ComposerResult};
use build_script_shared::parsers::{ParserDeserialize, ws, ParserSerialize};
use nom::sequence::terminated;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Dummy, Clone, Copy)]
pub enum Visibility {
    Public,
    Local,
    Private
}

impl<I: InputType> ParserDeserialize<I> for Visibility {
    fn parse(s: I) -> ParserResult<I, Self> {
        ws(map(
            opt(
                alt((
                    map(terminated(tag("pub"), multispace1), |_| Visibility::Public),
                    map(terminated(tag("local"), multispace1), |_| Visibility::Local)
                ))
            ),
            |o| o.unwrap_or_else(|| Visibility::Private)
        ))(s)
    }
}

impl ParserSerialize for Visibility {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        match self {
            Visibility::Private => write!(f, ""),
            Visibility::Public => write!(f, "pub "),
            Visibility::Local => write!(f, "local "),
        }?;
        Ok(())
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Private => write!(f, ""),
            Visibility::Public => write!(f, "pub"),
            Visibility::Local => write!(f, "local"),
        }?;

        Ok(())
    }
}

compose_test!{visibility_compose, Visibility}