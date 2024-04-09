use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::pair;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct AttributeFunctionKeyValue<I> {
    pub name: Ident<I>,
    pub key: Ident<I>,
    pub value: Ident<I>,
    #[serde(skip)]
    mark: Mark<I>,
}

impl<I> AttributeFunctionKeyValue<I> {
    pub fn new(name: Ident<I>, key: Ident<I>, value: Ident<I>, mark: Mark<I>) -> AttributeFunctionKeyValue<I> {
        AttributeFunctionKeyValue { 
            name, 
            key, 
            value, 
            mark 
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AttributeFunctionKeyValue<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        AttributeFunctionKeyValue {
            name: self.name.map(f),
            key: self.key.map(f),
            value: self.value.map(f),
            mark: self.mark.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for AttributeFunctionKeyValue<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ((name, (key, value)), mark)) = context(
            "Parsing AttributeFunctionKeyValue",
            marked(pair(
                Ident::ident,
                surrounded(
                    '(', 
                    key_value(
                        Ident::ident,
                        char('='),
                        Ident::ident
                    ),
                    ')'
                )
            )),
        )(s)?;

        Ok((s, AttributeFunctionKeyValue { 
            name,
            key,
            value,
            mark
        }))
    }
}

impl<I> ParserSerialize for AttributeFunctionKeyValue<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        _ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
        write!(f, "{}({} = {})", self.name, self.key, self.value)?;
        Ok(())
    }
}

impl<I> Marked<I> for AttributeFunctionKeyValue<I> {
    fn marker(&self) -> &Mark<I> {
        &self.mark
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for AttributeFunctionKeyValue<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        AttributeFunctionKeyValue {
            name: SimpleIdentDummy.fake_with_rng(rng),
            key: SimpleIdentDummy.fake_with_rng(rng),
            value: SimpleIdentDummy.fake_with_rng(rng),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

pub struct AllowedFunctionKeyValueAttribute(pub &'static [(&'static str, &'static str)]);
impl<I: Dummy<Faker>> Dummy<AllowedFunctionKeyValueAttribute> for AttributeFunctionKeyValue<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionKeyValueAttribute, rng: &mut R) -> Self {
        let (name, key) = config.0.choose(rng).unwrap();
        AttributeFunctionKeyValue {
            name: Ident::new(name.to_string(), Faker.fake_with_rng(rng)), 
            key: Ident::new(key.to_string(), Faker.fake_with_rng(rng)),
            value: SimpleIdentDummy.fake_with_rng(rng),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

compose_test! {attribute_key_value_compose_test, AttributeFunctionKeyValue<I>}
