use std::hash::Hash;
use crate::compose_test;
use crate::input_marker::InputType;
use crate::error::ParserResult;
use crate::parsers::*;
use nom::character::complete::*;
use nom::error::context;
use fake::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct AttributeKeyValue<I> {
    pub key: Ident<I>,
    pub value: Ident<I>,
    mark: Mark<I>
}

impl<I> AttributeKeyValue<I> {
    pub fn new(key: Ident<I>, value: Ident<I>, mark: Mark<I>) -> AttributeKeyValue<I> {
        AttributeKeyValue {
            key,
            value,
            mark
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AttributeKeyValue<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        AttributeKeyValue {
            key: self.key.map(f),
            value: self.value.map(f),
            mark: self.mark.map(f)
        }
    }
}

impl<I: InputType> ParserDeserialize<I>  for AttributeKeyValue<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ((key, value), mark)) = context(
                "Parsing AttributeKeyValue",
                marked(
                    key_value(Ident::ident, char('='), Ident::ident),
                )
        )(s)?;

        Ok((
            s,
            AttributeKeyValue { 
                key,
                value,
                mark
            }
        ))
    }
}

impl<I> ParserSerialize for AttributeKeyValue<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        write!(f, "{} = {}", self.key, self.value)?;
        Ok(())
    }
}

impl<I> Hash for AttributeKeyValue<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.value.hash(state);
    }
}

impl<I> PartialEq for AttributeKeyValue<I> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<I> Eq for AttributeKeyValue<I> {}

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
            mark: Faker.fake_with_rng(rng)
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
            mark: Faker.fake_with_rng(rng)
        }
    }
}

compose_test!{attribute_key_value_compose_test, AttributeKeyValue<I>}