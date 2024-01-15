use std::collections::BTreeMap;
use std::hash::Hash;
use crate::compose_test;
use crate::input_marker::InputType;
use crate::error::{ParserResult, ParserError, ParserSlimResult, ParserErrorKind};
use crate::parsers::*;
use nom::{multi::*, Err};
use nom::error::context;
use fake::*;

#[derive(Debug, Clone, Default, PartialOrd, Ord, Dummy)]
pub struct Attributes<I> {
    pub attributes: BTreeMap<usize, Attribute<I>>,
}

impl<I> Attributes<I> {
    pub fn new(attributes: BTreeMap<usize, Attribute<I>>) -> Self {
        Attributes {
            attributes
        }
    }

    pub fn get_key_value(&self, key: &str) -> Option<&AttributeKeyValue<I>> {
        self
            .iter()
            .filter_map(|attr| if let Attribute::KeyValue(value) = attr {
                Some(value)
            } else {
                None
            })
            .find(|attr| *attr.key == key)
    }

    pub fn get_functions(&self, key: &str) -> Vec<&AttributeFunction<I>> {
        self
            .iter()
            .filter_map(|attr| if let Attribute::Function(value) = attr {
                Some(value)
            } else {
                None
            })
            .filter(|attr| *attr.key == key)
            .collect()
    }

    pub fn check_key_value(&self, allowed_attributes: &[&str]) -> ParserSlimResult<I, ()> 
    where
        I: Clone
    {
        for attr in self.iter() {
            if let Attribute::KeyValue(attr) = attr {
                if !allowed_attributes.iter().any(|s| *attr.key == *s) {
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
        I: Clone
    {
        for attr in self.iter() {
            if let Attribute::Function(attr) = attr {
                if !allowed_attributes.iter().any(|(s, size)| *attr.key == *s && &attr.values.len() == size) {
                    return Err(Err::Failure(ParserError::new_at(
                        attr,
                        ParserErrorKind::InvalidAttribute(allowed_attributes
                            .iter()
                            .map(|(s, size)| format!("{}(*{})", s, size))
                            .collect::<Vec<_>>()
                            .join(", ")
                        ),
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Attribute<I>> {
        self.attributes.values()
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Attributes<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Attributes {
            attributes: self.attributes.into_iter().map(|(i, attr)| (i, attr.map(f))).collect()
        }
    }
}

impl<I: InputType> ParserDeserialize<I>  for Attributes<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, attributes) = context(
                "Parsing Attributes",
                many0(ws(Attribute::parse))
        )(s)?;

        Ok((
            s,
            Attributes { 
                attributes: attributes.into_iter().enumerate().collect()
            }
        ))
    }
}

impl<I> ParserSerialize for Attributes<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        let mut first = false;
        for (_, attribute) in &self.attributes {
            attribute.compose(f)?;
            if !first {
                writeln!(f, "")?;
            } else {
                first = false;
            }
        }
        Ok(())
    }
}

impl<I> Hash for Attributes<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (_, attribute) in &self.attributes {
            attribute.hash(state);
        }
    }
}

impl<I> PartialEq for Attributes<I> {
    fn eq(&self, other: &Self) -> bool {
        if self.attributes.len() != other.attributes.len() {
            return false;
        }

        let iter = self.attributes.iter().zip(other.attributes.iter());
        for ((_, attribute), (_, other_attribute)) in iter {
            if attribute != other_attribute {
                return false;
            }
        }

        true
    }
}

impl<I> Eq for Attributes<I> {}

const ATTRIBUTES_DUMMY_LENTGH: usize = 2;

impl<I: Dummy<Faker>> Dummy<AllowedAttributes> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedAttributes, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len).map(|i| (i, Attribute::dummy_with_rng(config, rng))).collect()
        }
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedKeyValueAttribute> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedKeyValueAttribute, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len).map(|i| (i, Attribute::dummy_with_rng(config, rng))).collect()
        }
    }
}

impl<I: Dummy<Faker>> Dummy<AllowedFunctionAttribute> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionAttribute, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len).map(|i| (i, Attribute::dummy_with_rng(config, rng))).collect()
        }
    }
}

compose_test!{attributes_compose_test, Attributes<I>}