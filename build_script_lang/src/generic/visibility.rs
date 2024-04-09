use std::fmt::Display;

use build_script_shared::error::{ComposerResult, ParserResult};
use build_script_shared::parsers::{ComposeContext, ParserDeserialize, ParserSerialize};
use build_script_shared::{compose_test, InputType};
use fake::Dummy;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::{map, opt};
use nom::sequence::terminated;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Dummy, Clone, Copy, Serialize, Deserialize,
)]
pub enum Visibility {
    Public,
    Local,
}

impl<I: InputType> ParserDeserialize<I> for Visibility {
    fn parse(s: I) -> ParserResult<I, Self> {
        map(
            opt(map(terminated(tag("pub"), multispace1), |_| {
                Visibility::Public
            })),
            |o| o.unwrap_or_else(|| Visibility::Local),
        )(s)
    }
}

impl ParserSerialize for Visibility {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        let indents = ctx.create_indents();
        match self {
            Visibility::Public => write!(f, "{indents}pub "),
            Visibility::Local => write!(f, "{indents}"),
        }?;
        Ok(())
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Local => write!(f, ""),
            Visibility::Public => write!(f, "pub"),
        }?;

        Ok(())
    }
}

compose_test! {visibility_compose, Visibility}
