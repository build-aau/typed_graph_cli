use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::character::complete::*;
use nom::error::context;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct AttributeKeyValue<I> {
    pub key: Ident<I>,
    pub value: Ident<I>,
    #[serde(skip)]
    mark: Mark<I>,
}

impl<I> AttributeKeyValue<I> {
    pub fn new(key: Ident<I>, value: Ident<I>, mark: Mark<I>) -> AttributeKeyValue<I> {
        AttributeKeyValue { key, value, mark }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AttributeKeyValue<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        AttributeKeyValue {
            key: self.key.map(f),
            value: self.value.map(f),
            mark: self.mark.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for AttributeKeyValue<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ((key, value), mark)) = context(
            "Parsing AttributeKeyValue",
            marked(key_value(Ident::ident, char('='), Ident::ident)),
        )(s)?;

        Ok((s, AttributeKeyValue { key, value, mark }))
    }
}

impl<I> ParserSerialize for AttributeKeyValue<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> crate::error::ComposerResult<()> {
        write!(f, "{} = {}", self.key, self.value)?;
        Ok(())
    }
}

impl<I> Marked<I> for AttributeKeyValue<I> {
    fn marker(&self) -> &Mark<I> {
        &self.mark
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for AttributeKeyValue<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        AttributeKeyValue {
            key: SimpleIdentDummy.fake_with_rng(rng),
            value: SimpleIdentDummy.fake_with_rng(rng),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

pub struct AllowedKeyValueAttribute(pub &'static [&'static str]);
impl<I: Dummy<Faker>> Dummy<AllowedKeyValueAttribute> for AttributeKeyValue<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedKeyValueAttribute, rng: &mut R) -> Self {
        let key = config.0.choose(rng).unwrap();
        AttributeKeyValue {
            key: Ident::new(key.to_string(), Faker.fake_with_rng(rng)),
            value: SimpleIdentDummy.fake_with_rng(rng),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

compose_test! {attribute_key_value_compose_test, AttributeKeyValue<I>}
