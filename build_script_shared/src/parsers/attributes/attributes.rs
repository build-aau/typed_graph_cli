use crate::compose_test;
use crate::error::{ParserError, ParserErrorKind, ParserResult, ParserSlimResult};
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::error::context;
use nom::{multi::*, Err};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Dummy, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct Attributes<I> {
    pub attributes: Vec<Attribute<I>>,
}

impl<I> Attributes<I> {
    pub fn new(attributes: Vec<Attribute<I>>) -> Self {
        Attributes { attributes }
    }

    pub fn get_key_value(&self, key: &str) -> Option<&AttributeKeyValue<I>> {
        self.iter()
            .filter_map(|attr| {
                if let Attribute::KeyValue(value) = attr {
                    Some(value)
                } else {
                    None
                }
            })
            .find(|attr| &*attr.key == key)
    }

    pub fn get_functions(&self, key: &str) -> Vec<&AttributeFunction<I>> {
        self.iter()
            .filter_map(|attr| {
                if let Attribute::Function(value) = attr {
                    Some(value)
                } else {
                    None
                }
            })
            .filter(|attr| &*attr.key == key)
            .collect()
    }

    pub fn check_key_value(&self, allowed_attributes: &[&str]) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for attr in self.iter() {
            if let Attribute::KeyValue(attr) = attr {
                if !allowed_attributes.iter().any(|s| &*attr.key == *s) {
                    return Err(Err::Failure(ParserError::new_at(
                        attr,
                        ParserErrorKind::InvalidAttribute(allowed_attributes.join(", ")),
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn check_function(&self, allowed_attributes: &[(&str, usize)]) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for attr in self.iter() {
            if let Attribute::Function(attr) = attr {
                if !allowed_attributes
                    .iter()
                    .any(|(s, size)| &*attr.key == *s && &attr.values.len() == size)
                {
                    return Err(Err::Failure(ParserError::new_at(
                        attr,
                        ParserErrorKind::InvalidAttribute(
                            allowed_attributes
                                .iter()
                                .map(|(s, size)| format!("{}(*{})", s, size))
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Attribute<I>> {
        self.attributes.iter()
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Attributes<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Attributes {
            attributes: self
                .attributes
                .into_iter()
                .map(|attr| attr.map(f))
                .collect(),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Attributes<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, attributes) = context("Parsing Attributes", many0(ws(Attribute::parse)))(s)?;

        Ok((
            s,
            Attributes {
                attributes
            },
        ))
    }
}

impl<I> ParserSerialize for Attributes<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> crate::error::ComposerResult<()> {
        let mut first = false;
        for attribute in &self.attributes {
            attribute.compose(f, ctx)?;
            if !first {
                writeln!(f, "")?;
            } else {
                first = false;
            }
        }
        Ok(())
    }
}

const ATTRIBUTES_DUMMY_LENTGH: usize = 2;

impl<I: Dummy<Faker>> Dummy<AllowedAttributes> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedAttributes, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len)
                .map(|_| Attribute::dummy_with_rng(config, rng))
                .collect(),
        }
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedKeyValueAttribute> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedKeyValueAttribute, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len)
                .map(|_| Attribute::dummy_with_rng(config, rng))
                .collect(),
        }
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedFunctionAttribute> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionAttribute, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len)
                .map(|_| Attribute::dummy_with_rng(config, rng))
                .collect(),
        }
    }
}

compose_test! {attributes_compose_test, Attributes<I>}
