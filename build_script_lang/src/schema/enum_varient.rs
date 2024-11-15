use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::{FieldWithReferences, Fields};
use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::{ParserError, ParserErrorKind, ParserSlimResult};
use build_script_shared::parsers::{
    marked, surrounded, ws, AllowedAttributes, AllowedFunctionAttribute,
    AllowedFunctionKeyValueAttribute, AllowedKeyValueAttribute, Attributes, Comments,
    ComposeContext, Ident, Mark, Marked, ParserDeserialize, ParserSerialize, TypeReferenceMap,
    Types,
};
use build_script_shared::{compose_test, InputType};
use fake::{Dummy, Fake, Faker};
use nom::branch::alt;
use nom::combinator::map;
use nom::combinator::success;
use nom::error::context;
use nom::Err;
use serde::{Deserialize, Serialize};

const JSON: &str = "json";
const DERIVE: &str = "derive";

const ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES: &[(&str, &str)] = &[(JSON, "alias")];

const ALLOWED_JSON_FUNCTION_ATTRIBUTE_VALUES: &[&str] = &["untagged"];

const ALLOWED_DERIVED_FUNCTION_ATTRIBUTE_VALUES: &[&str] = &["default"];

const ALLOWED_FUNCTION_ATTRIBUTES: &[(&str, Option<usize>, Option<&[&str]>)] = &[
    (JSON, Some(1), Some(ALLOWED_JSON_FUNCTION_ATTRIBUTE_VALUES)),
    (
        DERIVE,
        None,
        Some(ALLOWED_DERIVED_FUNCTION_ATTRIBUTE_VALUES),
    ),
];

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub enum EnumVarient<I> {
    // Quick note: I did implement Tuple enum and they worked
    // However there was no good way of making changesets.
    // if people made a new schema with a new field in the middle of the tuple
    // then it basicly invalidaded all the fields after it as there was no way of knowing
    // where to retrieve the previous versions fields from
    Struct {
        name: Ident<I>,
        #[serde(flatten)]
        attributes: Attributes<I>,
        comments: Comments,
        fields: Fields<I>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    Opaque {
        name: Ident<I>,
        #[serde(flatten)]
        attributes: Attributes<I>,
        comments: Comments,
        ty: Types<I>,
        #[serde(skip)]
        marker: Mark<I>,
    },
    Unit {
        #[serde(flatten)]
        attributes: Attributes<I>,
        name: Ident<I>,
        comments: Comments,
        #[serde(skip)]
        marker: Mark<I>,
    },
}

impl<I> EnumVarient<I> {
    pub fn name(&self) -> &Ident<I> {
        match self {
            EnumVarient::Struct { name, .. } => &name,
            EnumVarient::Unit { name, .. } => &name,
            EnumVarient::Opaque { name, .. } => &name,
        }
    }

    pub fn comments(&self) -> &Comments {
        match self {
            EnumVarient::Struct { comments, .. } => comments,
            EnumVarient::Unit { comments, .. } => comments,
            EnumVarient::Opaque { comments, .. } => comments,
        }
    }

    pub fn comments_mut(&mut self) -> &mut Comments {
        match self {
            EnumVarient::Struct { comments, .. } => comments,
            EnumVarient::Unit { comments, .. } => comments,
            EnumVarient::Opaque { comments, .. } => comments,
        }
    }

    pub fn attributes(&self) -> &Attributes<I> {
        match self {
            EnumVarient::Struct { attributes, .. } => attributes,
            EnumVarient::Unit { attributes, .. } => attributes,
            EnumVarient::Opaque { attributes, .. } => attributes,
        }
    }

    pub fn attributes_mut(&mut self) -> &mut Attributes<I> {
        match self {
            EnumVarient::Struct { attributes, .. } => attributes,
            EnumVarient::Unit { attributes, .. } => attributes,
            EnumVarient::Opaque { attributes, .. } => attributes,
        }
    }

    pub fn strip_comments(&mut self) {
        match self {
            EnumVarient::Struct {
                comments, fields, ..
            } => {
                comments.strip_comments();
                fields.strip_comments();
            }
            EnumVarient::Unit { comments, .. } => {
                comments.strip_comments();
            }
            EnumVarient::Opaque { comments, .. } => {
                comments.strip_comments();
            }
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EnumVarient<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            EnumVarient::Struct {
                attributes,
                name,
                comments,
                fields,
                marker,
            } => EnumVarient::Struct {
                attributes: attributes.map(f),
                name: name.map(f),
                comments,
                fields: fields.map(f),
                marker: marker.map(f),
            },
            EnumVarient::Unit {
                attributes,
                name,
                comments,
                marker,
            } => EnumVarient::Unit {
                attributes: attributes.map(f),
                name: name.map(f),
                comments,
                marker: marker.map(f),
            },
            EnumVarient::Opaque {
                attributes,
                name,
                comments,
                ty,
                marker,
            } => EnumVarient::Opaque {
                attributes: attributes.map(f),
                name: name.map(f),
                ty: ty.map(f),
                comments,
                marker: marker.map(f),
            },
        }
    }

    pub fn check_types(
        &self,
        reference_types: &HashMap<Ident<I>, Vec<String>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct { fields, .. } => fields.check_types(reference_types)?,
            EnumVarient::Opaque { ty, .. } => ty.check_types(reference_types)?,
            EnumVarient::Unit { .. } => (),
        };

        Ok(())
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct {
                attributes, fields, ..
            } => {
                EnumVarient::check_varient_attributes(attributes)?;
                fields.check_attributes()?;
            }
            EnumVarient::Opaque { attributes, .. } => {
                EnumVarient::check_varient_attributes(attributes)?;
            }
            EnumVarient::Unit { attributes, .. } => {
                EnumVarient::check_varient_attributes(attributes)?;
            }
        }
        Ok(())
    }

    fn check_varient_attributes(attributes: &Attributes<I>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        attributes.check_attributes(
            &[],
            ALLOWED_FUNCTION_ATTRIBUTES,
            ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES,
        )?;

        let json_functions = attributes.get_functions(JSON);
        for func in json_functions {
            if let Some(tag) = func.values.get(0) {
                if !ALLOWED_JSON_FUNCTION_ATTRIBUTE_VALUES.contains(&tag.as_str()) {
                    return Err(Err::Failure(ParserError::new_at(
                        tag,
                        ParserErrorKind::InvalidAttribute(format!(
                            "{}",
                            ALLOWED_JSON_FUNCTION_ATTRIBUTE_VALUES.join(",")
                        )),
                    )));
                }
            } else {
                return Err(Err::Failure(ParserError::new_at(
                    func,
                    ParserErrorKind::InvalidAttribute(format!("Expected 1 argument")),
                )));
            }
        }

        let derive_functions = attributes.get_functions(DERIVE);
        for func in derive_functions {
            if let Some(tag) = func.values.get(0) {
                if !ALLOWED_DERIVED_FUNCTION_ATTRIBUTE_VALUES.contains(&tag.as_str()) {
                    return Err(Err::Failure(ParserError::new_at(
                        tag,
                        ParserErrorKind::InvalidAttribute(format!(
                            "{}",
                            ALLOWED_DERIVED_FUNCTION_ATTRIBUTE_VALUES.join(",")
                        )),
                    )));
                }
            } else {
                return Err(Err::Failure(ParserError::new_at(
                    func,
                    ParserErrorKind::InvalidAttribute(format!("Expected 1 argument")),
                )));
            }
        }

        Ok(())
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
            EnumVarient::Struct { fields, .. } => {
                fields.check_cycle(type_name, type_generics, dependency_graph)?
            }
            EnumVarient::Opaque { ty, .. } => {
                ty.check_cycle(type_name, type_generics, dependency_graph)?
            }
            EnumVarient::Unit { .. } => (),
        };
        Ok(())
    }

    pub fn remove_used(&self, reference_types: &mut HashSet<Ident<I>>)
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct { fields, .. } => fields.remove_used(reference_types),
            EnumVarient::Opaque { ty, .. } => ty.remove_used(reference_types),
            EnumVarient::Unit { .. } => (),
        }
    }
}

impl<I> Marked<I> for EnumVarient<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            EnumVarient::Struct { marker, .. } => marker,
            EnumVarient::Opaque { marker, .. } => marker,
            EnumVarient::Unit { marker, .. } => marker,
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for EnumVarient<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, attributes) = Attributes::parse(s)?;
        let (s, (name, marker)) = context("Parsing Enum varient", ws(marked(Ident::ident)))(s)?;

        let (s, varient) = alt((
            map(
                context("Parsing Struct enum", Fields::parse),
                // Its not the prettiest thing that we clone here but it will do
                |fields| EnumVarient::Struct {
                    attributes: attributes.clone(),
                    name: name.clone(),
                    comments: comments.clone(),
                    fields,
                    marker: marker.clone(),
                },
            ),
            map(
                context("Parsing Struct enum", surrounded('(', Types::parse, ')')),
                // Its not the prettiest thing that we clone here but it will do
                |ty| EnumVarient::Opaque {
                    attributes: attributes.clone(),
                    name: name.clone(),
                    comments: comments.clone(),
                    ty,
                    marker: marker.clone(),
                },
            ),
            success(EnumVarient::Unit {
                attributes: attributes.clone(),
                name: name.clone(),
                comments: comments.clone(),
                marker: marker.clone(),
            }),
        ))(s)?;

        Ok((s, varient))
    }
}

impl<I> ParserSerialize for EnumVarient<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        match self {
            EnumVarient::Struct {
                name,
                comments,
                attributes,
                fields,
                ..
            } => {
                comments.compose(f, ctx)?;
                attributes.compose(f, ctx)?;
                name.compose(f, ctx)?;
                write!(f, " ")?;
                fields.compose(f, ctx)?;
            }
            EnumVarient::Opaque {
                name,
                comments,
                attributes,
                ty,
                ..
            } => {
                comments.compose(f, ctx)?;
                attributes.compose(f, ctx)?;
                name.compose(f, ctx)?;
                write!(f, " (")?;
                ty.compose(f, ctx)?;
                write!(f, ")")?;
            }
            EnumVarient::Unit {
                name,
                comments,
                attributes,
                ..
            } => {
                comments.compose(f, ctx)?;
                attributes.compose(f, ctx)?;
                name.compose(f, ctx)?;
            }
        }

        Ok(())
    }
}

pub struct EnumVarientOfType {
    pub name: String,
    pub ref_types: TypeReferenceMap,
}

impl<I: Dummy<Faker> + Clone> Dummy<EnumVarientOfType> for EnumVarient<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &EnumVarientOfType,
        rng: &mut R,
    ) -> Self {
        let mut varient = EnumVarient::dummy_with_rng(&Faker, rng);

        match &mut varient {
            EnumVarient::Struct { name, .. } => {
                *name = Ident::new(config.name.clone(), Mark::dummy_with_rng(&Faker, rng))
            }
            EnumVarient::Opaque { name, .. } => {
                *name = Ident::new(config.name.clone(), Mark::dummy_with_rng(&Faker, rng))
            }
            EnumVarient::Unit { name, .. } => {
                *name = Ident::new(config.name.clone(), Mark::dummy_with_rng(&Faker, rng))
            }
        };

        match &mut varient {
            EnumVarient::Struct { fields, .. } => {
                *fields =
                    Fields::dummy_with_rng(&FieldWithReferences(config.ref_types.clone()), rng);
            }
            EnumVarient::Opaque { ty, .. } => {
                config.ref_types.pick_valid_reference_type(ty, rng);
            }
            EnumVarient::Unit { .. } => (),
        }

        varient
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for EnumVarient<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let varient = rng.gen_range(0..3);
        match varient {
            0 => {
                let fields = Fields::dummy_with_rng(config, rng);

                if !fields.is_empty() {
                    EnumVarient::Struct {
                        attributes: AllowedAttributes(
                            AllowedKeyValueAttribute(&[]),
                            AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                            AllowedFunctionKeyValueAttribute(ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES),
                        )
                        .fake_with_rng(rng),
                        name: Dummy::dummy_with_rng(config, rng),
                        comments: Dummy::dummy_with_rng(config, rng),
                        fields,
                        marker: Dummy::dummy_with_rng(config, rng),
                    }
                } else {
                    EnumVarient::Unit {
                        attributes: AllowedAttributes(
                            AllowedKeyValueAttribute(&[]),
                            AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                            AllowedFunctionKeyValueAttribute(ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES),
                        )
                        .fake_with_rng(rng),
                        name: Dummy::dummy_with_rng(config, rng),
                        comments: Dummy::dummy_with_rng(config, rng),
                        marker: Dummy::dummy_with_rng(config, rng),
                    }
                }
            }
            1 => {
                let ty = Types::dummy_with_rng(config, rng);

                EnumVarient::Opaque {
                    attributes: AllowedAttributes(
                        AllowedKeyValueAttribute(&[]),
                        AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                        AllowedFunctionKeyValueAttribute(ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES),
                    )
                    .fake_with_rng(rng),
                    name: Dummy::dummy_with_rng(config, rng),
                    comments: Dummy::dummy_with_rng(config, rng),
                    ty,
                    marker: Dummy::dummy_with_rng(config, rng),
                }
            }
            _ => EnumVarient::Unit {
                attributes: AllowedAttributes(
                    AllowedKeyValueAttribute(&[]),
                    AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                    AllowedFunctionKeyValueAttribute(ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES),
                )
                .fake_with_rng(rng),
                name: Dummy::dummy_with_rng(config, rng),
                comments: Dummy::dummy_with_rng(config, rng),
                marker: Dummy::dummy_with_rng(config, rng),
            },
        }
    }
}

compose_test! {enum_varient_compose, EnumVarient<I>}
