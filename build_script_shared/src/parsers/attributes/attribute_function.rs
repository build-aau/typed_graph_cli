use std::collections::BTreeMap;
use std::hash::Hash;
use crate::compose_test;
use crate::input_marker::InputType;
use crate::error::ParserResult;
use crate::parsers::*;
use nom::sequence::*;
use nom::error::context;
use fake::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct AttributeFunction<I> {
    pub key: Ident<I>,
    pub values: BTreeMap<usize, Ident<I>>,
    mark: Mark<I>
}

impl<I> AttributeFunction<I> {
    pub fn new(key: Ident<I>, values: BTreeMap<usize, Ident<I>>, mark: Mark<I>) -> AttributeFunction<I> {
        AttributeFunction {
            key,
            values,
            mark
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AttributeFunction<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        AttributeFunction {
            key: self.key.map(f),
            values: self.values.into_iter().map(|(i, attr)| (i, attr.map(f))).collect(),
            mark: self.mark.map(f)
        }
    }
}

impl<I: InputType> ParserDeserialize<I>  for AttributeFunction<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ((key, values), mark)) = context(
            "Parsing AttributeFunction",
            marked(
                pair(
                    Ident::ident, 
                    surrounded(
                        '(', 
                        punctuated(Ident::ident, ','), 
                        ')'
                    )
                )
            )
        )(s)?;

        Ok((
            s,
            AttributeFunction { 
                key,
                values: values.into_iter().enumerate().collect(),
                mark
            }
        ))
    }
}

impl<I> ParserSerialize for AttributeFunction<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        write!(f, "{}(", self.key)?;
        let mut first = true;
        for value in self.values.values() {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            value.compose(f)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl<I> Hash for AttributeFunction<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.values.hash(state);
    }
}

impl<I> PartialEq for AttributeFunction<I> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.values == other.values
    }
}

impl<I> Eq for AttributeFunction<I> {}

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
            values: (0..len).map(|i| (i, SimpleIdentDummy.fake_with_rng(rng))).collect(),
            mark: Faker.fake_with_rng(rng)
        }
    }
}

pub struct AllowedFunctionAttribute(pub &'static [(&'static str, usize)]);
impl<I: Dummy<Faker>> Dummy<AllowedFunctionAttribute> for AttributeFunction<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionAttribute, rng: &mut R) -> Self {
        let (key, len) = config.0.choose(rng).unwrap();
        AttributeFunction {
            key: Ident::new(key.to_string(), Faker.fake_with_rng(rng)),
            values: (0..*len).map(|i| (i, SimpleIdentDummy.fake_with_rng(rng))).collect(),
            mark: Faker.fake_with_rng(rng)
        }
    }
}

compose_test!{attribute_function_compose_test, AttributeFunction<I>}
