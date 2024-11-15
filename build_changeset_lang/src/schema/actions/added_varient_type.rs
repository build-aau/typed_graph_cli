use std::fmt::Display;

use build_script_lang::schema::EnumVarient;
use build_script_shared::parsers::{
    surrounded, ComposeContext, ParserDeserialize, ParserSerialize, Types,
};
use build_script_shared::{compose_test, InputType};
use fake::Dummy;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, value};
use nom::error::context;
use nom::sequence::preceded;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub enum AddedVarientType<I> {
    Struct,
    Opaque(Types<I>),
    Unit,
}

impl<I> AddedVarientType<I> {
    pub fn map<O, F>(self, f: F) -> AddedVarientType<O>
    where
        F: Fn(I) -> O + Copy,
    {
        match self {
            AddedVarientType::Struct => AddedVarientType::Struct,
            AddedVarientType::Unit => AddedVarientType::Unit,
            AddedVarientType::Opaque(ty) => AddedVarientType::Opaque(ty.map(f)),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedVarientType<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        context(
            "Parsing AddedVarientType",
            alt((
                value(AddedVarientType::Struct, tag("struct")),
                map(
                    preceded(tag("opaque"), surrounded('(', Types::parse, ')')),
                    AddedVarientType::Opaque,
                ),
                value(AddedVarientType::Unit, tag("unit")),
            )),
        )(s)
    }
}

impl<I> ParserSerialize for AddedVarientType<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        _ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        match self {
            AddedVarientType::Struct => write!(f, "struct")?,
            AddedVarientType::Opaque(ty) => {
                write!(f, "opaque(")?;
                ty.compose(f, _ctx)?;
                write!(f, ")")?;
            }
            AddedVarientType::Unit => write!(f, "unit")?,
        }

        Ok(())
    }
}

impl<I> Display for AddedVarientType<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

impl<I: Clone> From<&EnumVarient<I>> for AddedVarientType<I> {
    fn from(value: &EnumVarient<I>) -> Self {
        match value {
            EnumVarient::Struct { .. } => AddedVarientType::Struct,
            EnumVarient::Opaque { ty, .. } => AddedVarientType::Opaque(ty.clone()),
            EnumVarient::Unit { .. } => AddedVarientType::Unit,
        }
    }
}

compose_test! {added_varient_type_compose, AddedVarientType<I>}
