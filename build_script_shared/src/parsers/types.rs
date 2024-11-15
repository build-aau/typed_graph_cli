use super::*;
use crate::compose_test;
use crate::dependency_graph::DependencyGraph;
use crate::error::{ParserError, ParserErrorKind, ParserResult, ParserSlimResult};
use crate::input_marker::InputType;
use fake::{Dummy, Faker, Rng};
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::char;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::{pair, preceded, terminated};
use nom::Err;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
#[serde(tag = "type")]
pub enum Types<I> {
    String(#[serde(skip)] Mark<I>),
    #[serde(rename = "bool")]
    Bool(#[serde(skip)] Mark<I>),
    #[serde(rename = "f64")]
    F64(#[serde(skip)] Mark<I>),
    #[serde(rename = "f32")]
    F32(#[serde(skip)] Mark<I>),
    #[serde(rename = "usize")]
    Usize(#[serde(skip)] Mark<I>),
    #[serde(rename = "u64")]
    U64(#[serde(skip)] Mark<I>),
    #[serde(rename = "u32")]
    U32(#[serde(skip)] Mark<I>),
    #[serde(rename = "u16")]
    U16(#[serde(skip)] Mark<I>),
    #[serde(rename = "u8")]
    U8(#[serde(skip)] Mark<I>),
    #[serde(rename = "isize")]
    Isize(#[serde(skip)] Mark<I>),
    #[serde(rename = "i64")]
    I64(#[serde(skip)] Mark<I>),
    #[serde(rename = "i32")]
    I32(#[serde(skip)] Mark<I>),
    #[serde(rename = "i16")]
    I16(#[serde(skip)] Mark<I>),
    #[serde(rename = "i8")]
    I8(#[serde(skip)] Mark<I>),
    Option {
        inner: Box<Types<I>>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    List {
        inner: Box<Types<I>>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    Set {
        inner: Box<Types<I>>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    Map {
        key: Box<Types<I>>,
        value: Box<Types<I>>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    Reference {
        inner: Ident<I>,
        generics: Vec<Box<Types<I>>>,
        #[serde(skip)]
        marker: Mark<I>,
    },
}

impl<I> Types<I> {
    pub fn check_types(
        &self,
        reference_types: &HashMap<Ident<I>, Vec<String>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            Types::String(_)
            | Types::Usize(_)
            | Types::Bool(_)
            | Types::F64(_)
            | Types::F32(_)
            | Types::U64(_)
            | Types::U32(_)
            | Types::U16(_)
            | Types::U8(_)
            | Types::Isize(_)
            | Types::I64(_)
            | Types::I32(_)
            | Types::I16(_)
            | Types::I8(_) => Ok(()),
            Types::Option { inner, .. } => inner.check_types(reference_types),
            Types::List { inner, .. } => inner.check_types(reference_types),
            Types::Set { inner, .. } => inner.check_types(reference_types),
            Types::Map { key, value, .. } => key
                .check_types(reference_types)
                .and_then(|_| value.check_types(reference_types)),
            Types::Reference {
                inner,
                generics,
                marker,
            } => {
                if let Some(ty_generics) = reference_types.get(inner) {
                    let expected_generic_count = ty_generics.len();
                    let actual_generic_count = generics.len();
                    if actual_generic_count < expected_generic_count {
                        return Err(Err::Failure(ParserError::new_at(
                            marker,
                            ParserErrorKind::UnexpectedGenericCount(
                                inner.to_string(),
                                expected_generic_count,
                                actual_generic_count,
                            ),
                        )));
                    }

                    if actual_generic_count > expected_generic_count {
                        return Err(Err::Failure(ParserError::new_at(
                            marker,
                            ParserErrorKind::UnexpectedGenericCount(
                                inner.to_string(),
                                expected_generic_count,
                                actual_generic_count,
                            ),
                        )));
                    }
                } else {
                    return Err(Err::Failure(ParserError::new_at(
                        inner,
                        ParserErrorKind::UnknownReference(inner.to_string()),
                    )));
                }

                for generic in generics {
                    generic.check_types(reference_types)?;
                }

                Ok(())
            }
        }
    }

    pub fn check_cycle<'a>(
        &'a self,
        type_name: &'a Ident<I>,
        type_generics: &Vec<String>,
        dependency_graph: &mut DependencyGraph<'a, I>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            Types::String(_)
            | Types::Usize(_)
            | Types::Bool(_)
            | Types::F64(_)
            | Types::F32(_)
            | Types::U64(_)
            | Types::U32(_)
            | Types::U16(_)
            | Types::U8(_)
            | Types::Isize(_)
            | Types::I64(_)
            | Types::I32(_)
            | Types::I16(_)
            | Types::I8(_) => Ok(()),
            Types::Option { inner, .. } => {
                inner.check_cycle(type_name, type_generics, dependency_graph)
            }
            Types::List { inner, .. } => {
                inner.check_cycle(type_name, type_generics, dependency_graph)
            }
            Types::Set { inner, .. } => {
                inner.check_cycle(type_name, type_generics, dependency_graph)
            }
            Types::Map { key, value, .. } => {
                key.check_cycle(type_name, type_generics, dependency_graph)?;
                value.check_cycle(type_name, type_generics, dependency_graph)?;
                Ok(())
            }
            Types::Reference {
                inner, generics, ..
            } => {
                if type_generics.contains(&inner.to_string()) {
                    return Ok(());
                }

                if dependency_graph.contains(inner) && dependency_graph.contains(type_name) {
                    dependency_graph.add_dependency(type_name, inner)?;

                    // Everytime we specify the value of a generic
                    // We add it as a dependency
                    for generic in generics {
                        if !type_generics.contains(&generic.to_string()) {
                            generic.check_cycle(type_name, type_generics, dependency_graph)?;
                        }
                    }

                    Ok(())
                } else {
                    Err(Err::Failure(ParserError::new_at(
                        inner,
                        ParserErrorKind::OwnedContext(format!("Failed to resolve cyclic graph as {inner} or {type_name} is not initalized")),
                    )))
                }
            }
        }
    }

    pub fn remove_used(&self, reference_types: &mut HashSet<Ident<I>>) {
        match self {
            Types::String(_)
            | Types::Usize(_)
            | Types::Bool(_)
            | Types::F64(_)
            | Types::F32(_)
            | Types::U64(_)
            | Types::U32(_)
            | Types::U16(_)
            | Types::U8(_)
            | Types::Isize(_)
            | Types::I64(_)
            | Types::I32(_)
            | Types::I16(_)
            | Types::I8(_) => (),
            Types::Option { inner, .. } => inner.remove_used(reference_types),
            Types::List { inner, .. } => inner.remove_used(reference_types),
            Types::Set { inner, .. } => inner.remove_used(reference_types),
            Types::Map { key, value, .. } => {
                key.remove_used(reference_types);
                value.remove_used(reference_types);
            }
            Types::Reference {
                inner, generics, ..
            } => {
                reference_types.remove(inner);

                for generic in generics {
                    generic.remove_used(reference_types);
                }
            }
        }
    }

    pub fn map_reference<F>(self, f: F) -> Self
    where
        F: Fn(Ident<I>) -> Ident<I> + Copy,
    {
        match self {
            Types::String(s) => Types::String(s),
            Types::Bool(s) => Types::Bool(s),
            Types::F64(s) => Types::F64(s),
            Types::F32(s) => Types::F32(s),
            Types::Usize(s) => Types::Usize(s),
            Types::U64(s) => Types::U64(s),
            Types::U32(s) => Types::U32(s),
            Types::U16(s) => Types::U16(s),
            Types::U8(s) => Types::U8(s),
            Types::Isize(s) => Types::Isize(s),
            Types::I64(s) => Types::I64(s),
            Types::I32(s) => Types::I32(s),
            Types::I16(s) => Types::I16(s),
            Types::I8(s) => Types::I8(s),
            Types::Option { inner, marker } => Types::Option {
                inner: inner.map_reference(f).into(),
                marker,
            },
            Types::List { inner, marker } => Types::List {
                inner: inner.map_reference(f).into(),
                marker,
            },
            Types::Set { inner, marker } => Types::Set {
                inner: inner.map_reference(f).into(),
                marker,
            },
            Types::Map { key, value, marker } => Types::Map {
                key: key.map_reference(f).into(),
                value: value.map_reference(f).into(),
                marker,
            },
            Types::Reference {
                inner,
                generics,
                marker,
            } => Types::Reference {
                inner: f(inner),
                generics: generics
                    .into_iter()
                    .map(|generic| generic.map_reference(f).into())
                    .collect(),
                marker,
            },
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Types<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            Types::String(s) => Types::String(s.map(f)),
            Types::Bool(s) => Types::Bool(s.map(f)),
            Types::F64(s) => Types::F64(s.map(f)),
            Types::F32(s) => Types::F32(s.map(f)),
            Types::Usize(s) => Types::Usize(s.map(f)),
            Types::U64(s) => Types::U64(s.map(f)),
            Types::U32(s) => Types::U32(s.map(f)),
            Types::U16(s) => Types::U16(s.map(f)),
            Types::U8(s) => Types::U8(s.map(f)),
            Types::Isize(s) => Types::Isize(s.map(f)),
            Types::I64(s) => Types::I64(s.map(f)),
            Types::I32(s) => Types::I32(s.map(f)),
            Types::I16(s) => Types::I16(s.map(f)),
            Types::I8(s) => Types::I8(s.map(f)),
            Types::Option { inner, marker } => Types::Option {
                inner: inner.map(f).into(),
                marker: marker.map(f),
            },
            Types::List { inner, marker } => Types::List {
                inner: inner.map(f).into(),
                marker: marker.map(f),
            },
            Types::Set { inner, marker } => Types::Set {
                inner: inner.map(f).into(),
                marker: marker.map(f),
            },
            Types::Map { key, value, marker } => Types::Map {
                key: key.map(f).into(),
                value: value.map(f).into(),
                marker: marker.map(f),
            },
            Types::Reference {
                inner,
                generics,
                marker,
            } => Types::Reference {
                inner: inner.map(f),
                generics: generics.into_iter().map(|g| g.map(f).into()).collect(),
                marker: marker.map(f),
            },
        }
    }

    pub fn check_convertion(&self, other: &Types<I>) -> bool {
        match (self, other) {
            (Types::String(_), Types::String(_))
            | (Types::Bool(_), Types::Bool(_))
            | (Types::Usize(_), Types::Usize(_))
            | (Types::Isize(_), Types::Isize(_))
            // f32
            | (Types::F32(_), Types::F32(_))
            | (Types::F32(_), Types::F64(_))
            // f64
            | (Types::F64(_), Types::F64(_))
            // u8
            | (Types::U8(_), Types::U8(_))
            | (Types::U8(_), Types::U16(_))
            | (Types::U8(_), Types::U32(_))
            | (Types::U8(_), Types::U64(_))
            // u16
            | (Types::U16(_), Types::U16(_))
            | (Types::U16(_), Types::U32(_))
            | (Types::U16(_), Types::U64(_))
            // u32
            | (Types::U32(_), Types::U32(_))
            | (Types::U32(_), Types::U64(_))
            // u64
            | (Types::U64(_), Types::U64(_))
            // i8
            | (Types::I8(_), Types::I8(_))
            | (Types::I8(_), Types::I16(_))
            | (Types::I8(_), Types::I32(_))
            | (Types::I8(_), Types::I64(_))
            // i16
            | (Types::I16(_), Types::I16(_))
            | (Types::I16(_), Types::I32(_))
            | (Types::I16(_), Types::I64(_))
            // i32
            | (Types::I32(_), Types::I32(_))
            | (Types::I32(_), Types::I64(_))
            // i64
            | (Types::I64(_), Types::I64(_)) => true,

            // Reference types only works if their inner types can be converted
            (Types::Option{inner: linner, .. }, Types::Option{inner: rinner, .. })
            | (Types::List{inner: linner, .. }, Types::List{inner: rinner, .. })
            | (Types::Set{inner: linner, .. }, Types::Set{inner: rinner, .. })
            | (Types::List{inner: linner, .. }, Types::Set{inner: rinner, .. })
            | (Types::Set{inner: linner, .. }, Types::List{inner: rinner, .. }) => {
                linner.check_convertion(&rinner)
            }
            (
                Types::Map{key: lkey, value: lvalue, ..},
                Types::Map{key: rkey, value: rvalue, ..}
            ) => {
                lkey.check_convertion(&rkey) && lvalue.check_convertion(&rvalue)
            }

            (t, Types::Option { inner, .. }) => {
                t.check_convertion(inner)
            }

            // All convertion of external types are left entirely to the user to handler
            (_, Types::Reference { .. })
            | (Types::Reference { .. }, _) => true,
            _ => false
        }
    }

    pub fn check_convertion_res(&self, other: &Types<I>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let is_valid_convertion = self.check_convertion(other);

        if is_valid_convertion {
            Ok(())
        } else {
            // Find the type responsible for not allowing convertion
            match (self, other) {
                (Types::Option { inner: linner, .. }, Types::Option { inner: rinner, .. })
                | (Types::List { inner: linner, .. }, Types::List { inner: rinner, .. })
                | (Types::Set { inner: linner, .. }, Types::Set { inner: rinner, .. })
                | (Types::Set { inner: linner, .. }, Types::List { inner: rinner, .. })
                | (Types::List { inner: linner, .. }, Types::Set { inner: rinner, .. }) => {
                    let res = linner.check_convertion_res(&rinner);
                    if res.is_err() {
                        return res;
                    }
                }
                (t, Types::Option { inner, .. }) => {
                    let res = t.check_convertion_res(&inner);
                    if res.is_err() {
                        return res;
                    }
                }
                (
                    Types::Map {
                        key: lkey,
                        value: lvalue,
                        ..
                    },
                    Types::Map {
                        key: rkey,
                        value: rvalue,
                        ..
                    },
                ) => {
                    let res = lkey
                        .check_convertion_res(&rkey)
                        .and(lvalue.check_convertion_res(&rvalue));
                    if res.is_err() {
                        return res;
                    }
                }
                _ => (),
            }

            // If no specific type could be found we just point at the current one
            Err(Err::Failure(ParserError::new_at(
                other,
                ParserErrorKind::InvalidTypeConvertion(self.to_string(), other.to_string()),
            )))
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Types<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        context(
            "Parsing Types",
            alt((
                map(
                    marked(terminated(
                        preceded(tag("Option"), surrounded('<', cut(Types::parse), '>')),
                        not(Ident::ident_full),
                    )),
                    |(ty, marker)| Types::Option {
                        inner: ty.into(),
                        marker,
                    },
                ),
                map(
                    marked(terminated(
                        preceded(tag("List"), surrounded('<', cut(Types::parse), '>')),
                        not(Ident::ident_full),
                    )),
                    |(ty, marker)| Types::List {
                        inner: ty.into(),
                        marker,
                    },
                ),
                map(
                    marked(terminated(
                        preceded(tag("Set"), surrounded('<', cut(Types::parse), '>')),
                        not(Ident::ident_full),
                    )),
                    |(ty, marker)| Types::Set {
                        inner: ty.into(),
                        marker,
                    },
                ),
                map(
                    marked(terminated(
                        preceded(
                            tag("Map"),
                            surrounded(
                                '<',
                                cut(key_value(Types::parse, char(','), Types::parse)),
                                '>',
                            ),
                        ),
                        not(Ident::ident_full),
                    )),
                    |((kty, vty), marker)| Types::Map {
                        key: kty.into(),
                        value: vty.into(),
                        marker,
                    },
                ),
                map(
                    marked(terminated(tag("String"), not(Ident::ident_full))),
                    |(_, marker)| Types::String(marker),
                ),
                map(
                    marked(terminated(tag("bool"), not(Ident::ident_full))),
                    |(_, marker)| Types::Bool(marker),
                ),
                map(
                    marked(terminated(tag("f64"), not(Ident::ident_full))),
                    |(_, marker)| Types::F64(marker),
                ),
                map(
                    marked(terminated(tag("f32"), not(Ident::ident_full))),
                    |(_, marker)| Types::F32(marker),
                ),
                map(
                    marked(terminated(tag("usize"), not(Ident::ident_full))),
                    |(_, marker)| Types::Usize(marker),
                ),
                map(
                    marked(terminated(tag("u64"), not(Ident::ident_full))),
                    |(_, marker)| Types::U64(marker),
                ),
                map(
                    marked(terminated(tag("u32"), not(Ident::ident_full))),
                    |(_, marker)| Types::U32(marker),
                ),
                map(
                    marked(terminated(tag("u16"), not(Ident::ident_full))),
                    |(_, marker)| Types::U16(marker),
                ),
                map(
                    marked(terminated(tag("u8"), not(Ident::ident_full))),
                    |(_, marker)| Types::U8(marker),
                ),
                map(
                    marked(terminated(tag("isize"), not(Ident::ident_full))),
                    |(_, marker)| Types::Isize(marker),
                ),
                map(
                    marked(terminated(tag("i64"), not(Ident::ident_full))),
                    |(_, marker)| Types::I64(marker),
                ),
                map(
                    marked(terminated(tag("i32"), not(Ident::ident_full))),
                    |(_, marker)| Types::I32(marker),
                ),
                map(
                    marked(terminated(tag("i16"), not(Ident::ident_full))),
                    |(_, marker)| Types::I16(marker),
                ),
                map(
                    marked(terminated(tag("i8"), not(Ident::ident_full))),
                    |(_, marker)| Types::I8(marker),
                ),
                map(
                    marked(pair(
                        Ident::ident,
                        opt(surrounded('<', punctuated(Types::parse, ','), '>')),
                    )),
                    |((inner, generics), marker)| Types::Reference {
                        inner,
                        generics: generics
                            .unwrap_or_default()
                            .into_iter()
                            .map(Into::into)
                            .collect(),
                        marker,
                    },
                ),
            )),
        )(s)
    }
}

impl<I> ParserSerialize for Types<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
        match self {
            Types::String(_) => write!(f, "String")?,
            Types::Bool(_) => write!(f, "bool")?,
            Types::F64(_) => write!(f, "f64")?,
            Types::F32(_) => write!(f, "f32")?,
            Types::Usize(_) => write!(f, "usize")?,
            Types::U64(_) => write!(f, "u64")?,
            Types::U32(_) => write!(f, "u32")?,
            Types::U16(_) => write!(f, "u16")?,
            Types::U8(_) => write!(f, "u8")?,
            Types::Isize(_) => write!(f, "isize")?,
            Types::I64(_) => write!(f, "i64")?,
            Types::I32(_) => write!(f, "i32")?,
            Types::I16(_) => write!(f, "i16")?,
            Types::I8(_) => write!(f, "i8")?,
            Types::Option { inner, .. } => write!(f, "Option<{inner}>")?,
            Types::List { inner, .. } => write!(f, "List<{inner}>")?,
            Types::Set { inner, .. } => write!(f, "Set<{inner}>")?,
            Types::Map { key, value, .. } => write!(f, "Map<{key}, {value}>")?,
            Types::Reference {
                inner, generics, ..
            } => {
                inner.compose(f, ctx)?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    let mut first = true;
                    for generic in generics {
                        if !first {
                            write!(f, ", ")?;
                        } else {
                            first = false;
                        }
                        generic.compose(f, ctx)?;
                    }
                    write!(f, ">")?;
                }
            }
        };
        Ok(())
    }
}

impl<I> PartialEq for Types<I> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Types::String(_), Types::String(_))
            | (Types::Bool(_), Types::Bool(_))
            | (Types::F64(_), Types::F64(_))
            | (Types::F32(_), Types::F32(_))
            | (Types::Usize(_), Types::Usize(_))
            | (Types::U64(_), Types::U64(_))
            | (Types::U32(_), Types::U32(_))
            | (Types::U16(_), Types::U16(_))
            | (Types::U8(_), Types::U8(_))
            | (Types::Isize(_), Types::Isize(_))
            | (Types::I64(_), Types::I64(_))
            | (Types::I32(_), Types::I32(_))
            | (Types::I16(_), Types::I16(_))
            | (Types::I8(_), Types::I8(_)) => true,

            (Types::Option { inner: inner0, .. }, Types::Option { inner: inner1, .. })
            | (Types::List { inner: inner0, .. }, Types::List { inner: inner1, .. })
            | (Types::Set { inner: inner0, .. }, Types::Set { inner: inner1, .. }) => {
                inner0.eq(inner1)
            }
            (
                Types::Map {
                    key: key0,
                    value: value0,
                    ..
                },
                Types::Map {
                    key: key1,
                    value: value1,
                    ..
                },
            ) => key0.eq(key1) && value0.eq(value1),
            (
                Types::Reference {
                    inner: inner0,
                    generics: generics0,
                    ..
                },
                Types::Reference {
                    inner: inner1,
                    generics: generics1,
                    ..
                },
            ) => inner0.eq(inner1) && generics0.eq(generics1),
            _ => false,
        }
    }
}

impl<I> Eq for Types<I> {}

impl<I> Display for Types<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::String(_) => write!(f, "String"),
            Types::Bool(_) => write!(f, "bool"),
            Types::F64(_) => write!(f, "f64"),
            Types::F32(_) => write!(f, "f32"),
            Types::Usize(_) => write!(f, "usize"),
            Types::U64(_) => write!(f, "u64"),
            Types::U32(_) => write!(f, "u32"),
            Types::U16(_) => write!(f, "u16"),
            Types::U8(_) => write!(f, "u8"),
            Types::Isize(_) => write!(f, "isize"),
            Types::I64(_) => write!(f, "i64"),
            Types::I32(_) => write!(f, "i32"),
            Types::I16(_) => write!(f, "i16"),
            Types::I8(_) => write!(f, "i8"),
            Types::Option { inner, .. } => write!(f, "Option<{inner}>"),
            Types::List { inner, .. } => write!(f, "List<{inner}>"),
            Types::Set { inner, .. } => write!(f, "Set<{inner}>"),
            Types::Map { key, value, .. } => write!(f, "Map<{key}, {value}>"),
            Types::Reference {
                inner, generics, ..
            } => {
                write!(f, "{inner}")?;
                if !generics.is_empty() {
                    write!(f, "<")?;
                    let mut first = true;
                    for generic in generics {
                        if !first {
                            write!(f, ",")?;
                        } else {
                            first = false;
                        }
                        write!(f, "{generic}")?;
                    }
                    write!(f, ">")?;
                }

                Ok(())
            }
        }
    }
}

impl<I> Marked<I> for Types<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            Types::String(marker)
            | Types::Bool(marker)
            | Types::F64(marker)
            | Types::F32(marker)
            | Types::Usize(marker)
            | Types::U64(marker)
            | Types::U32(marker)
            | Types::U16(marker)
            | Types::U8(marker)
            | Types::Isize(marker)
            | Types::I64(marker)
            | Types::I32(marker)
            | Types::I16(marker)
            | Types::I8(marker)
            | Types::Option { marker, .. }
            | Types::List { marker, .. }
            | Types::Set { marker, .. }
            | Types::Map { marker, .. }
            | Types::Reference { marker, .. } => marker,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeReferenceMap(pub HashMap<String, usize>);

impl TypeReferenceMap {
    /// alter all references to so their types are compatible with the list of available types
    pub fn pick_valid_reference_type<I: Dummy<Faker>, R: rand::prelude::Rng + ?Sized>(
        &self,
        ty: &mut Types<I>,
        rng: &mut R,
    ) {
        match ty {
            Types::String(_)
            | Types::Bool(_)
            | Types::F64(_)
            | Types::F32(_)
            | Types::Usize(_)
            | Types::U64(_)
            | Types::U32(_)
            | Types::U16(_)
            | Types::U8(_)
            | Types::Isize(_)
            | Types::I64(_)
            | Types::I32(_)
            | Types::I16(_)
            | Types::I8(_) => (),
            Types::Option { inner, .. }
            | Types::List { inner, .. }
            | Types::Set { inner, .. } => self.pick_valid_reference_type(inner, rng),
            Types::Map { key, value, .. } => {
                self.pick_valid_reference_type(key, rng);
                self.pick_valid_reference_type(value, rng);
            }
            Types::Reference {
                inner, generics, ..
            } => {
                if let Some(ref_type) = self.0.iter().choose(rng) {
                    let (name, generic_count) = ref_type;
                    *inner = Ident::new(name, Mark::dummy_with_rng(&Faker, rng));

                    while generics.len() > *generic_count {
                        generics.pop();
                    }

                    while generics.len() < *generic_count {
                        generics.push(Types::dummy_with_rng(&Faker, rng).into());
                    }

                    for generic in generics {
                        self.pick_valid_reference_type(generic, rng);
                    }
                } else {
                    *ty = Types::Bool(Mark::dummy_with_rng(&Faker, rng));
                }
            }
        }
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for Types<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
        match rng.gen_range(0..5) {
            0 => Types::String(Mark::dummy_with_rng(&Faker, rng)),
            1 => match rng.gen_range(0..2) {
                0 => Types::F64(Mark::dummy_with_rng(&Faker, rng)),
                1 | _ => Types::F32(Mark::dummy_with_rng(&Faker, rng)),
            },
            2 => match rng.gen_range(0..10) {
                0 => Types::Usize(Mark::dummy_with_rng(&Faker, rng)),
                1 => Types::U64(Mark::dummy_with_rng(&Faker, rng)),
                2 => Types::U32(Mark::dummy_with_rng(&Faker, rng)),
                3 => Types::U16(Mark::dummy_with_rng(&Faker, rng)),
                4 => Types::U8(Mark::dummy_with_rng(&Faker, rng)),
                5 => Types::Isize(Mark::dummy_with_rng(&Faker, rng)),
                6 => Types::I64(Mark::dummy_with_rng(&Faker, rng)),
                7 => Types::I32(Mark::dummy_with_rng(&Faker, rng)),
                8 => Types::I16(Mark::dummy_with_rng(&Faker, rng)),
                9 | _ => Types::I8(Mark::dummy_with_rng(&Faker, rng)),
            },
            3 => match rng.gen_range(0..5) {
                0 => Types::Map {
                    key: Box::new(Types::dummy_with_rng(&Faker, rng)),
                    value: Box::new(Types::dummy_with_rng(&Faker, rng)),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
                1 => Types::List {
                    inner: Box::new(Types::dummy_with_rng(&Faker, rng)),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
                2 => Types::Set {
                    inner: Box::new(Types::dummy_with_rng(&Faker, rng)),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
                3 => Types::Option {
                    inner: Box::new(Types::dummy_with_rng(&Faker, rng)),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
                4 | _ => Types::Reference {
                    inner: Ident::dummy_with_rng(&Faker, rng),
                    generics: (0..3)
                        .map(|_| Box::new(Types::dummy_with_rng(&Faker, rng)))
                        .collect(),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
            },
            4 | _ => Types::Bool(Mark::dummy_with_rng(&Faker, rng)),
        }
    }
}

impl Deref for TypeReferenceMap {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TypeReferenceMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

compose_test! {types_compose, Types<I>}
