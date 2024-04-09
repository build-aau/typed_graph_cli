use std::collections::HashMap;
use std::collections::HashSet;

use super::FieldValue;
use super::{FieldWithReferences, Fields};
use build_script_shared::compose_test;
use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::ParserError;
use build_script_shared::error::ParserErrorKind;
use build_script_shared::error::ParserResult;
use build_script_shared::error::ParserSlimResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Fake;
use fake::Faker;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::*;
use nom::multi::*;
use nom::sequence::*;
use nom::Err;
use serde::Deserialize;
use serde::Serialize;

const DERIVE: &str = "derive";
const JSON: &str = "json";

const JSON_ATTRIBUTES: &[&'static str] = &[
    "untagged"
];

const ALLOWED_FUNCTION_ATTRIBUTES: &[(&str, Option<usize>)] = &[
    (DERIVE, None),
    (JSON, Some(1)),
];

#[derive(
    PartialEq, Eq, Debug, Hash, Clone, Default, PartialOrd, Ord, Dummy, Serialize, Deserialize,
)]
#[serde(bound = "I: Default + Clone")]
pub struct StructExp<I> {
    pub name: Ident<I>,
    #[dummy(faker = "AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES)")]
    #[serde(flatten)]
    pub attributes: Attributes<I>,
    #[serde(flatten)]
    pub comments: Comments,
    #[serde(flatten)]
    pub generics: Generics<I>,
    #[serde(flatten)]
    pub fields: Fields<I>,
    #[serde(skip)]
    marker: Mark<I>,
}

impl<I> StructExp<I> {
    pub fn new(
        comments: Comments,
        attributes: Attributes<I>,
        name: Ident<I>,
        generics: Generics<I>,
        fields: Fields<I>,
        marker: Mark<I>,
    ) -> Self {
        StructExp {
            attributes,
            comments,
            name,
            generics,
            fields,
            marker,
        }
    }

    pub fn strip_comments(&mut self) {
        self.comments.strip_comments();
        self.fields.strip_comments();
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> StructExp<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        StructExp {
            attributes: self.attributes.map(f),
            generics: self.generics.map(f),
            comments: self.comments,
            name: self.name.map(f),
            fields: self.fields.map(f),
            marker: self.marker.map(f),
        }
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.attributes.check_attributes(
            &[], 
            ALLOWED_FUNCTION_ATTRIBUTES, 
            &[]
        )?;

        let json_functions = self.attributes.get_functions(JSON);
        for func in json_functions {
            if let Some(tag) = func.values.get(0) {
                if !JSON_ATTRIBUTES.contains(&tag.as_str()) {
                    return Err(Err::Failure(ParserError::new_at(tag, ParserErrorKind::InvalidAttribute(format!("{}", JSON_ATTRIBUTES.join(","))))));
                }
            } else {
                return Err(Err::Failure(ParserError::new_at(func, ParserErrorKind::InvalidAttribute(format!("Expected 1 argument")))));
            }
        }

        self.fields.check_attributes()?;

        Ok(())
    }

    pub fn check_types(
        &self,
        reference_types: &HashMap<Ident<I>, Vec<String>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let mut local_references = reference_types.clone();
        for generic in &self.generics.generics {
            local_references.insert(generic.letter.clone(), Default::default());
        }

        self.fields.check_types(&local_references)
    }

    pub fn check_cycle<'a>(
        &'a self,
        dependency_graph: &mut DependencyGraph<'a, I>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let type_generics = self.generics.get_meta();
        self.fields
            .check_cycle(&self.name, &type_generics, dependency_graph)
    }

    pub fn check_used(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let mut local_references = HashSet::new();
        for generic in &self.generics.generics {
            local_references.insert(generic.letter.clone());
        }

        self.fields.remove_used(&mut local_references);

        for generic in &self.generics.generics {
            if local_references.contains(&generic.letter) {
                return Err(Err::Failure(ParserError::new_at(
                    &generic.letter,
                    ParserErrorKind::UnusedGeneric,
                )));
            }
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for StructExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, attributes) = Attributes::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("struct"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context("Parsing Struct type", ws(cut(marked(Ident::ident))))(s)?;
        let (s, generics) = Generics::parse(s)?;
        // Parse the list of fields
        let (s, fields) = owned_context(format!("Parsing {}", name), cut(opt(Fields::parse)))(s)?;

        Ok((
            s,
            StructExp {
                attributes,
                generics,
                comments,
                name,
                fields: fields.unwrap_or_default(),
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for StructExp<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        self.comments.compose(f, ctx)?;
        self.attributes.compose(f, ctx)?;
        write!(f, "{indents}struct ")?;
        self.name.compose(f, ctx.set_indents(0))?;
        write!(f, " ")?;
        self.generics.compose(f, ctx.set_indents(0))?;
        self.fields.compose(f, ctx)?;

        Ok(())
    }
}

impl<I> Marked<I> for StructExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

pub(crate) struct StructExpOfType<I> {
    pub name: Ident<I>,
    pub generic_count: usize,
    pub ref_types: TypeReferenceMap,
}

impl<I: Dummy<Faker> + Clone> Dummy<StructExpOfType<I>> for StructExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &StructExpOfType<I>,
        rng: &mut R,
    ) -> Self {
        let generics = Generics::dummy_with_rng(&GenericsOfSize(config.generic_count), rng);

        // Create a list of all valid references
        let mut local_ref_types = config.ref_types.clone();
        for generic in &generics.generics {
            local_ref_types.insert(generic.letter.to_string(), 0);
        }

        let mut fields = Fields::dummy_with_rng(&FieldWithReferences(local_ref_types), rng);

        // Test if all the generics are referenced in the fields
        let mut all_generics = generics.generics.iter().map(|g| g.letter.clone()).collect();

        fields.remove_used(&mut all_generics);

        // Add phantom data for any of the missing generics
        for generic in &generics.generics {
            if !all_generics.contains(&generic.letter) {
                continue;
            }

            fields.insert_field(FieldValue {
                name: Ident::new(
                    format!("phantom_{}", generic.letter),
                    Mark::dummy_with_rng(&Faker, rng),
                ),
                attributes: AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES).fake_with_rng(rng),
                visibility: Dummy::dummy_with_rng(&Faker, rng),
                comments: Dummy::dummy_with_rng(&Faker, rng),
                field_type: Types::Reference {
                    inner: generic.letter.clone(),
                    generics: Default::default(),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                },
                order: fields.last_order().map_or_else(|| 0, |order| order + 1),
            });
        }

        StructExp {
            attributes: Attributes::dummy_with_rng(&AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES), rng),
            name: config.name.clone(),
            comments: Dummy::dummy_with_rng(&Faker, rng),
            generics,
            fields,
            marker: Dummy::dummy_with_rng(&Faker, rng),
        }
    }
}

compose_test! {struct_compose, StructExp<I>}
