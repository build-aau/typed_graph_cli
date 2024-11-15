use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::error::context;
use nom::sequence::*;
use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct AttributeFunction<I> {
    pub key: Ident<I>,
    pub values: Vec<Ident<I>>,
    #[serde(skip)]
    mark: Mark<I>,
}

impl<I> AttributeFunction<I> {
    pub fn new(key: Ident<I>, values: Vec<Ident<I>>, mark: Mark<I>) -> AttributeFunction<I> {
        AttributeFunction { key, values, mark }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AttributeFunction<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        AttributeFunction {
            key: self.key.map(f),
            values: self.values.into_iter().map(|a| a.map(f)).collect(),
            mark: self.mark.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for AttributeFunction<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ((key, values), mark)) = context(
            "Parsing AttributeFunction",
            marked(pair(
                Ident::ident,
                surrounded('(', punctuated(Ident::ident, ','), ')'),
            )),
        )(s)?;

        Ok((s, AttributeFunction { key, values, mark }))
    }
}

impl<I> ParserSerialize for AttributeFunction<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
        write!(f, "{}(", self.key)?;
        let mut first = true;
        let value_ctx = ctx.set_indents(0);
        for value in &self.values {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            value.compose(f, value_ctx)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl<I> Marked<I> for AttributeFunction<I> {
    fn marker(&self) -> &Mark<I> {
        &self.mark
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for AttributeFunction<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..10);
        AttributeFunction {
            key: SimpleIdentDummy.fake_with_rng(rng),
            values: (0..len)
                .map(|_| SimpleIdentDummy.fake_with_rng(rng))
                .collect(),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

pub struct AllowedFunctionAttribute(
    pub &'static [(&'static str, Option<usize>, Option<&'static [&'static str]>)],
);
impl<I: Dummy<Faker>> Dummy<AllowedFunctionAttribute> for AttributeFunction<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionAttribute, rng: &mut R) -> Self {
        let (key, len, allowed_values) = config.0.choose(rng).unwrap();
        AttributeFunction {
            key: Ident::new(key.to_string(), Faker.fake_with_rng(rng)),
            values: (0..len.unwrap_or_else(|| 5))
                .map(|_| {
                    let a = allowed_values
                        .and_then(|values| {
                            values
                                .iter()
                                .choose(rng)
                                .map(|value| Ident::new(value, Faker.fake()))
                        })
                        .unwrap_or_else(|| SimpleIdentDummy.fake_with_rng(rng));
                    a
                })
                .collect(),
            mark: Faker.fake_with_rng(rng),
        }
    }
}

compose_test! {attribute_function_compose_test, AttributeFunction<I>}
