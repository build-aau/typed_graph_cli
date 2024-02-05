use std::fmt::Display;

use build_script_lang::schema::EnumVarient;
use build_script_shared::{compose_test, InputType};
use build_script_shared::parsers::{ComposeContext, ParserDeserialize, ParserSerialize};
use fake::Dummy;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::error::context;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub enum AddedVarientType {
    Struct,
    Unit
}

impl<I: InputType> ParserDeserialize<I> for AddedVarientType {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        context(
            "Parsing AddedVarientType",
            alt((
                value(AddedVarientType::Struct, tag("struct")),
                value(AddedVarientType::Unit, tag("unit")),
            )),
        )(s)
    }
}

impl ParserSerialize for AddedVarientType {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        match self {
            AddedVarientType::Struct => write!(f, "struct")?,
            AddedVarientType::Unit => write!(f, "unit")?,
        }

        Ok(())
    }
}

impl Display for AddedVarientType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

impl<I> From<&EnumVarient<I>> for AddedVarientType {
    fn from(value: &EnumVarient<I>) -> Self {
        match value {
            EnumVarient::Struct { .. } => AddedVarientType::Struct,
            EnumVarient::Unit { .. } => AddedVarientType::Unit,
        }
    }
}

compose_test! {added_varient_type_compose, AddedVarientType}