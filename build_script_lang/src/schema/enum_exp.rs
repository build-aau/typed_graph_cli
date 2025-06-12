use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::{compose_test, InputType};
use fake::Dummy;
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
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use super::EnumVarient;
use super::EnumVarientOfType;
use super::FieldValue;
use super::Fields;

const DERIVE: &str = "derive";
const JSON: &str = "json";

const JSON_ATTRIBUTES: &[&'static str] = &["untagged"];

const ALLOWED_FUNCTION_ATTRIBUTES: &[(&str, Option<usize>, Option<&[&str]>)] =
    &[(DERIVE, None, None), (JSON, Some(1), Some(JSON_ATTRIBUTES))];

#[derive(PartialEq, Eq, Debug, Clone, Default, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct EnumExp<I> {
    pub name: Ident<I>,
    #[serde(flatten)]
    pub attributes: Attributes<I>,
    #[serde(flatten)]
    pub generics: Generics<I>,
    #[serde(flatten)]
    pub comments: Comments,
    pub varients: Vec<EnumVarient<I>>,
    #[serde(skip)]
    marker: Mark<I>,
}

impl<'c, I> EnumExp<I> {
    pub fn new(
        comments: Comments,
        attributes: Attributes<I>,
        name: Ident<I>,
        generics: Generics<I>,
        varients: Vec<EnumVarient<I>>,
        marker: Mark<I>,
    ) -> Self {
        EnumExp {
            attributes,
            comments,
            name,
            generics,
            varients,
            marker,
        }
    }

    pub fn is_only_units(&self) -> bool {
        let mut is_safe = true;

        for varient in &self.varients {
            match varient {
                EnumVarient::Unit { .. } => (),
                EnumVarient::Opaque { .. } | EnumVarient::Struct { .. } => {
                    is_safe = false;
                }
            }
        }

        is_safe
    }

    pub fn strip_comments(&mut self) {
        self.comments.strip_comments();

        for varient in &mut self.varients {
            varient.strip_comments();
        }
    }

    pub fn has_varient<S>(&self, varient_name: S) -> bool
    where
        S: for<'a> PartialEq<&'a Ident<I>>,
    {
        self.varient_position(varient_name).is_some()
    }

    pub fn varient_position<S>(&self, varient_name: S) -> Option<usize>
    where
        S: for<'a> PartialEq<&'a Ident<I>>,
    {
        self.varients
            .iter()
            .position(|varient| varient_name == varient.name())
    }

    pub fn get_varient<S>(&self, varient_name: S) -> Option<&EnumVarient<I>>
    where
        S: for<'a> PartialEq<&'a Ident<I>>,
    {
        self.varients
            .iter()
            .find(|varient| varient_name == varient.name())
    }

    pub fn get_varient_mut<S>(&mut self, varient_name: S) -> Option<&mut EnumVarient<I>>
    where
        S: for<'a> PartialEq<&'a Ident<I>>,
    {
        let i = self.varient_position(varient_name)?;
        self.varients.get_mut(i)
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EnumExp<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        EnumExp {
            attributes: self.attributes.map(f),
            comments: self.comments,
            name: self.name.map(f),
            generics: self.generics.map(f),
            varients: self
                .varients
                .into_iter()
                .map(|varient| varient.map(f))
                .collect(),
            marker: self.marker.map(f),
        }
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

        for varient in &self.varients {
            varient.check_types(&local_references)?;
        }
        Ok(())
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.attributes
            .check_attributes(&[], ALLOWED_FUNCTION_ATTRIBUTES, &[])?;

        for attrr in self.attributes.iter() {
            match attrr {
                Attribute::Function(attr) => {
                    if attr.key == JSON {
                        for value in &attr.values {
                            if !JSON_ATTRIBUTES.contains(&value.as_ref()) {
                                return Err(Err::Failure(
                                    ParserError::new_at(
                                        attr,
                                        ParserErrorKind::InvalidAttribute(format!(
                                            "Failed to match {value} as a valid attribute allowed ones are {}", 
                                            JSON_ATTRIBUTES.join(", ")
                                        ))
                                    )
                                ));
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        for varient in &self.varients {
            varient.check_attributes()?;
        }

        Ok(())
    }

    pub fn check_cycle<'a>(
        &'a self,
        dependency_graph: &mut DependencyGraph<'a, I>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let generics = self.generics.get_meta();
        for varient in &self.varients {
            varient.check_cycle(&self.name, &generics, dependency_graph)?;
        }
        Ok(())
    }

    pub fn check_used(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let mut local_references = HashSet::new();
        for generic in &self.generics.generics {
            local_references.insert(generic.letter.clone());
        }

        for varient in &self.varients {
            varient.remove_used(&mut local_references);
        }

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

    pub fn has_external_ref(&self) -> bool {
        let mut has_external_ref = false;
        for varient in &self.varients {
            has_external_ref |= varient.has_external_ref();
        }
        has_external_ref
    }
}

impl<I: InputType> ParserDeserialize<I> for EnumExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, attributes) = Attributes::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("enum"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context("Parsing Enum type", ws(cut(marked(Ident::ident))))(s)?;
        let (s, generics) = Generics::parse(s)?;
        // Parse the list of fields
        let (s, varients) = owned_context(
            format!("Parsing {}", name),
            cut(surrounded('{', punctuated(EnumVarient::parse, ','), '}')),
        )(s)?;

        let mut varients_checker: BTreeSet<&Ident<I>> = BTreeSet::new();
        for varient in &varients {
            if !varients_checker.insert(varient.name()) {
                let first = varients_checker
                    .iter()
                    .find(|v| varient.name().cmp(v).is_eq())
                    .cloned();
                let first = first.unwrap();

                return Err(Err::Failure(
                    vec![
                        (
                            varient.marker(),
                            ParserErrorKind::DuplicateDefinition(varient.name().to_string()),
                        ),
                        (first.marker(), ParserErrorKind::FirstOccurance),
                    ]
                    .into_iter()
                    .collect(),
                ));
            }
        }

        Ok((
            s,
            EnumExp {
                attributes,
                comments,
                name,
                generics,
                varients,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for EnumExp<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        let indents = ctx.create_indents();

        self.comments.compose(f, ctx)?;
        self.attributes.compose(f, ctx)?;
        write!(f, "{indents}enum ")?;
        self.name.compose(f, ctx)?;
        self.generics.compose(f, ctx.set_indents(0))?;
        writeln!(f, " {{")?;
        let varient_iter = self.varients.iter().enumerate();
        let mut first = true;
        for (_, varient) in varient_iter {
            if !first {
                writeln!(f, ",")?;
            } else {
                first = false;
            }
            varient.compose(f, ctx.increment_indents(1))?;
        }
        if !first {
            writeln!(f)?;
        }
        write!(f, "{indents}}}")?;
        Ok(())
    }
}

impl<I> Marked<I> for EnumExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I: Hash> Hash for EnumExp<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.comments.hash(state);
        self.name.hash(state);
        self.varients.hash(state);
        self.marker.hash(state);
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for EnumExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..10);

        // Ensure all the varients has unique names
        let mut taken_names = HashSet::new();
        let mut varient_names = Vec::new();

        let mut i = 0;
        while i < len {
            let new_name: Ident<I> = Ident::dummy_with_rng(&Faker, rng);
            if taken_names.contains(&*new_name) {
                continue;
            }

            taken_names.insert(new_name.to_string());
            varient_names.push(new_name);
            i += 1;
        }

        // Update the varient names
        let varients: Vec<_> = varient_names
            .into_iter()
            .map(|varient_name| {
                let mut varient = EnumVarient::dummy_with_rng(&Faker, rng);

                match &mut varient {
                    EnumVarient::Struct { name, .. } => *name = varient_name,
                    EnumVarient::Opaque { name, .. } => *name = varient_name,
                    EnumVarient::Unit { name, .. } => *name = varient_name,
                };

                varient
            })
            .collect();

        let exp = EnumExp {
            attributes: Attributes::dummy_with_rng(
                &AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                rng,
            ),
            name: Ident::dummy_with_rng(&Faker, rng),
            comments: Comments::dummy_with_rng(&Faker, rng),
            generics: Generics::dummy_with_rng(&Faker, rng),
            varients,
            marker: Mark::dummy_with_rng(&Faker, rng),
        };

        exp
    }
}

pub(crate) struct EnumExpOfType<I> {
    pub name: Ident<I>,
    pub generic_count: usize,
    pub ref_types: TypeReferenceMap,
}

impl<I: Dummy<Faker> + Clone> Dummy<EnumExpOfType<I>> for EnumExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &EnumExpOfType<I>,
        rng: &mut R,
    ) -> Self {
        let generics = Generics::dummy_with_rng(&GenericsOfSize(config.generic_count), rng);
        let mut local_ref_types = config.ref_types.clone();
        for generic in &generics.generics {
            local_ref_types.insert(generic.letter.to_string(), 0);
        }

        // test if all generics are being referenced
        let mut all_generics = generics
            .generics
            .iter()
            .map(|g| Ident::new(g.letter.to_string(), Mark::dummy_with_rng(&Faker, rng)))
            .collect();

        let mut exp = EnumExp {
            attributes: Attributes::dummy_with_rng(
                &AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES),
                rng,
            ),
            name: config.name.clone(),
            generics,
            comments: Dummy::dummy_with_rng(&Faker, rng),
            varients: (0..10)
                .map(|_| {
                    let varient_config = EnumVarientOfType {
                        name: Ident::<I>::dummy_with_rng(&Faker, rng).to_string(),
                        ref_types: local_ref_types.clone(),
                    };
                    let varient = EnumVarient::dummy_with_rng(&varient_config, rng);

                    if let EnumVarient::Struct { fields, .. } = &varient {
                        fields.remove_used(&mut all_generics);
                    }

                    varient
                })
                .collect(),
            marker: Mark::dummy_with_rng(&Faker, rng),
        };

        // Add phantom data to capture unreferenced generics
        if !all_generics.is_empty() {
            let mut phantom_fields =
                Fields::new(Default::default(), Mark::dummy_with_rng(&Faker, rng));
            for generic in &exp.generics.generics {
                if !all_generics.contains(&generic.letter) {
                    continue;
                }

                let mut value = FieldValue::dummy_with_rng(&Faker, rng);

                value.name = Ident::new(
                    format!("Phantom{}", generic.letter),
                    Mark::dummy_with_rng(&Faker, rng),
                );
                value.field_type = Types::Reference {
                    inner: Ident::new(
                        generic.letter.to_string(),
                        Mark::dummy_with_rng(&Faker, rng),
                    ),
                    generics: Default::default(),
                    marker: Mark::dummy_with_rng(&Faker, rng),
                };
                value.order = phantom_fields
                    .last_order()
                    .map_or_else(|| 0, |order| order + 1);

                phantom_fields.insert_field(value);
            }

            let phantom_varient = EnumVarient::Struct {
                attributes: Attributes::new(Default::default()),
                name: Ident::<I>::new("Phantom".to_string(), Mark::dummy_with_rng(&Faker, rng)),
                comments: Dummy::dummy_with_rng(&Faker, rng),
                fields: phantom_fields,
                _marker: Mark::dummy_with_rng(&Faker, rng),
            };

            exp.varients.push(phantom_varient);
        }

        exp
    }
}

compose_test! {enum_compose, EnumExp<I>}
