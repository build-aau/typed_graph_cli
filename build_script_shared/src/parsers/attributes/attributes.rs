use crate::compose_test;
use crate::error::{ParserError, ParserErrorKind, ParserResult, ParserSlimResult};
use crate::input_marker::InputType;
use crate::parsers::*;
use fake::*;
use nom::error::context;
use nom::{multi::*, Err};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(
    Debug, Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Dummy, Serialize, Deserialize,
)]
#[serde(bound = "I: Default + Clone")]
pub struct Attributes<I> {
    pub attributes: Vec<Attribute<I>>,
}

impl<I> Attributes<I> {
    pub fn new(attributes: Vec<Attribute<I>>) -> Self {
        Attributes { attributes }
    }

    pub fn is_skipped(&self) -> bool {
        self
            .get_functions("json")
            .iter()
            .any(|attr| attr.values.iter().any(|v| *v == "skip"))
    }

    pub fn is_untagged(&self) -> bool {
        self
            .get_functions("json")
            .iter()
            .any(|attr| attr.values.iter().any(|v| *v == "untagged"))
    }

    pub fn get_alias(&self) -> Vec<&Ident<I>> {
        self
            .get_key_value_functions("json")
            .into_iter()
            .filter(|kv| *kv.key == "alias")
            .map(|kv| &kv.value)
            .collect()
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

    pub fn get_key_value_functions(&self, key: &str) -> Vec<&AttributeFunctionKeyValue<I>> {
        self.iter()
            .filter_map(|attr| {
                if let Attribute::FunctionKeyValue(value) = attr {
                    Some(value)
                } else {
                    None
                }
            })
            .filter(|attr| &*attr.name == key)
            .collect()
    }

    pub fn check_attributes(
        &self,
        allow_key_value: &[&str],
        allowed_functions: &[(&str, Option<usize>)],
        allow_function_key_value: &[(&str, &str)]
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for attr in self.iter() {
            let allow_attribute = match attr {
                Attribute::Function(attr) => {
                    allowed_functions
                        .iter()
                        .any(|(s, size)| *attr.key == *s && size.map_or_else(|| true, |s| attr.values.len() == s))
                },
                Attribute::KeyValue(attr) => {
                    allow_key_value.iter().any(|name| *attr.key == *name)
                },
                Attribute::FunctionKeyValue(attr) => {
                    allow_function_key_value.iter().any(|(name, key)| *attr.key == *key && *attr.name == *name)
                }
            };

            if !allow_attribute {
                let f_attr = allowed_functions
                    .iter()
                    .map(|(s, size)| format!("{}(*{:?})", s, size))
                    .collect::<Vec<_>>()
                    .join(", ");

                let kv_attr = allow_key_value
                    .iter()
                    .map(|name| format!("{}=?", name))
                    .collect::<Vec<_>>()
                    .join(", ");

                let f_kv_attr = allow_function_key_value
                    .iter()
                    .map(|(name, key)| format!("{}({}=?)", name, key))
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(Err::Failure(ParserError::new_at(
                    attr,
                    ParserErrorKind::InvalidAttribute(format!("{}|{}|{}", kv_attr, f_attr, f_kv_attr)),
                )));
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

        Ok((s, Attributes { attributes }))
    }
}

impl<I> ParserSerialize for Attributes<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
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

impl<I: Dummy<Faker>> Dummy<AllowedFunctionKeyValueAttribute> for Attributes<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AllowedFunctionKeyValueAttribute, rng: &mut R) -> Self {
        let len = rng.gen_range(0..ATTRIBUTES_DUMMY_LENTGH);

        Attributes {
            attributes: (0..len)
                .map(|_| Attribute::dummy_with_rng(config, rng))
                .collect(),
        }
    }
}

compose_test! {attributes_compose_test, Attributes<I>}
