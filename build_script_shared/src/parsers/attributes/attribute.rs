use super::{AttributeFunction, AttributeKeyValue};
use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::branch::alt;
use nom::character::complete::*;
use nom::combinator::map;
use nom::error::context;
use nom::sequence::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
#[serde(tag = "type")]
pub enum Attribute<I> {
    KeyValue(AttributeKeyValue<I>),
    Function(AttributeFunction<I>),
}

impl<I> Attribute<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Attribute<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            Attribute::KeyValue(kv) => Attribute::KeyValue(kv.map(f)),
            Attribute::Function(value) => Attribute::Function(value.map(f)),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Attribute<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        context(
            "Partin Attribute",
            preceded(
                char('@'),
                surrounded(
                    '(',
                    alt((
                        map(AttributeKeyValue::parse, Attribute::KeyValue),
                        map(AttributeFunction::parse, Attribute::Function),
                    )),
                    ')',
                ),
            ),
        )(s)
    }
}

impl<I> ParserSerialize for Attribute<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> crate::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        write!(f, "{indents}@(")?;
        match self {
            Attribute::Function(value) => value.compose(f, ctx)?,
            Attribute::KeyValue(value) => value.compose(f, ctx)?,
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl<I> Marked<I> for Attribute<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            Attribute::Function(value) => value.marker(),
            Attribute::KeyValue(value) => value.marker(),
        }
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for Attribute<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        match rng.gen_range(0..=1) {
            0 => Attribute::KeyValue(AttributeKeyValue::dummy_with_rng(config, rng)),
            _ => Attribute::Function(AttributeFunction::dummy_with_rng(config, rng)),
        }
    }
}

pub struct AllowedAttributes(pub AllowedKeyValueAttribute, pub AllowedFunctionAttribute);
impl<I: Dummy<Faker>> Dummy<AllowedAttributes> for Attribute<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedAttributes, rng: &mut R) -> Self {
        match rng.gen_range(0..2) {
            0 => Attribute::KeyValue(AttributeKeyValue::dummy_with_rng(&config.0, rng)),
            _ => Attribute::Function(AttributeFunction::dummy_with_rng(&config.1, rng)),
        }
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedKeyValueAttribute> for Attribute<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedKeyValueAttribute, rng: &mut R) -> Self {
        Attribute::KeyValue(AttributeKeyValue::dummy_with_rng(config, rng))
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedFunctionAttribute> for Attribute<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionAttribute, rng: &mut R) -> Self {
        Attribute::Function(AttributeFunction::dummy_with_rng(config, rng))
    }
}

compose_test! {attribute_compose_test, Attribute<I>}
