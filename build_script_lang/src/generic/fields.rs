use build_script_shared::compose_test;
use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Faker;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::pair;
use nom::sequence::tuple;
use nom::Err;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use super::Visibility;

const JSON: &str = "json";

const ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES: &[(&str, &str)] = &[
    (JSON, "alias"), 
];

const ALLOWED_FUNCTION_ATTRIBUTE_VALUES: &[&str] = &[
    "skip"
];

const ALLOWED_FUNCTION_ATTRIBUTES: &[(&str, Option<usize>)] = &[
    (JSON, Some(1)),
];

#[derive(Debug, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct Fields<I> {
    fields: Vec<FieldValue<I>>,
    #[serde(skip)]
    marker: Mark<I>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Dummy, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct FieldValue<I> {
    pub name: Ident<I>,
    pub visibility: Visibility,
    #[dummy(faker = "AllowedAttributes(
        AllowedKeyValueAttribute(&[]),
        AllowedFunctionAttribute(ALLOWED_FUNCTION_ATTRIBUTES), 
        AllowedFunctionKeyValueAttribute(ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES)
    )")]
    pub attributes: Attributes<I>,
    #[serde(flatten)]
    pub comments: Comments,
    pub field_type: Types<I>,
    /// The order in which the field should be shown.  
    /// Having multiple fields on the same order is undefined behaviour
    #[serde(skip)]
    pub order: u64,
}

impl<I> Fields<I> {
    pub fn new(fields: Vec<FieldValue<I>>, marker: Mark<I>) -> Fields<I> {
        Fields { fields, marker }
    }

    pub fn strip_comments(&mut self) {
        for field in &mut self.fields {
            field.comments.strip_comments();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &FieldValue<I>> {
        let mut fields: Vec<_> = self.fields.iter().collect();
        fields.sort_by_key(|v| v.order);
        fields.into_iter()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn has_field<'a, S: PartialEq<&'a str>>(&'a self, field_name: S) -> bool {
        self.fields
            .iter()
            .any(|field| field_name == field.name.as_str())
    }

    pub fn remove_field(&mut self, field_name: &Ident<I>) -> Option<FieldValue<I>> {
        let remove_idx = self
            .fields
            .iter()
            .position(|field| &field.name == field_name);

        if let Some(idx) = remove_idx {
            Some(self.fields.remove(idx))
        } else {
            None
        }
    }

    pub fn get_field<'a, S: PartialEq<&'a str>>(
        &'a self,
        field_name: S,
    ) -> Option<&'a FieldValue<I>> {
        self.fields.iter().find(|field| field_name == &field.name)
    }

    pub fn get_field_mut<S>(&mut self, field_name: S) -> Option<&mut FieldValue<I>>
    where
        S: for<'a> PartialEq<&'a str>,
    {
        self.fields
            .iter_mut()
            .find(|field| field_name == &field.name)
    }

    pub fn insert_field(&mut self, field_values: FieldValue<I>) {
        self.fields.push(field_values);
    }

    pub fn last_order(&self) -> Option<u64> {
        self.fields.iter().map(|v| v.order).max()
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for field_value in &self.fields {
            field_value.attributes.check_attributes(
                &[], 
                ALLOWED_FUNCTION_ATTRIBUTES, 
                ALLOWED_FUNCTION_KEY_VALUE_ATTRIBUTES
            )?;

            let json_functions = field_value.attributes.get_functions(JSON);
            for func in json_functions {
                if let Some(tag) = func.values.get(0) {
                    if !ALLOWED_FUNCTION_ATTRIBUTE_VALUES.contains(&tag.as_str()) {
                        return Err(Err::Failure(ParserError::new_at(tag, ParserErrorKind::InvalidAttribute(format!("{}", ALLOWED_FUNCTION_ATTRIBUTE_VALUES.join(","))))));
                    }
                } else {
                    return Err(Err::Failure(ParserError::new_at(func, ParserErrorKind::InvalidAttribute(format!("Expected 1 argument")))));
                }
            }
        }

        Ok(())
    }

    pub fn check_types(
        &self,
        reference_types: &HashMap<Ident<I>, Vec<String>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for field_value in &self.fields {
            field_value.field_type.check_types(reference_types)?;
        }

        Ok(())
    }

    /// Check if a field have a given type
    ///
    /// If the type is empty, then the fields type is inserted
    pub fn check_field_type<'a>(
        &'a self,
        field: &str,
        id_type: &mut Option<(&'a Types<I>, String)>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        if let Some(field_value) = self.get_field(field) {
            if let Some((ty_origin, ty)) = &id_type {
                if ty != &field_value.field_type.to_string() {
                    return Err(Err::Failure(
                        vec![
                            (
                                field_value.name.marker(),
                                ParserErrorKind::UnexpectedFieldType(
                                    field.to_string(),
                                    field_value.field_type.to_string(),
                                ),
                            ),
                            (ty_origin.marker(), ParserErrorKind::FirstOccurance),
                        ]
                        .into_iter()
                        .collect(),
                    ));
                }
            } else {
                *id_type = Some((&field_value.field_type, field_value.field_type.to_string()));
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
        for field_value in &self.fields {
            field_value
                .field_type
                .check_cycle(type_name, type_generics, dependency_graph)?;
        }

        Ok(())
    }

    pub fn remove_used(&self, reference_types: &mut HashSet<Ident<I>>) {
        for field_value in &self.fields {
            field_value.field_type.remove_used(reference_types);
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Fields<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Fields {
            fields: self
                .fields
                .into_iter()
                .map(|field| FieldValue {
                    name: field.name.map(f),
                    visibility: field.visibility,
                    attributes: field.attributes.map(f),
                    comments: field.comments,
                    field_type: field.field_type.map(f),
                    order: field.order,
                })
                .collect(),
            marker: self.marker.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Fields<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (fields_raw, marker)) = context(
            "Parsing Fields",
            marked(surrounded(
                '{',
                punctuated(
                    tuple((
                        Comments::parse,
                        Attributes::parse,
                        key_value(
                            pair(Visibility::parse, Ident::ident),
                            char(':'),
                            Types::parse,
                        ),
                    )),
                    ',',
                ),
                '}',
            )),
        )(s)?;

        // Populate the list of fields
        let mut fields: Vec<FieldValue<I>> = Vec::new();
        let fields_iter = fields_raw.into_iter().enumerate();
        for (order, (comments, attributes, ((visibility, name), ty))) in fields_iter {
            let first_occurance = fields.iter().find(|field| &field.name == &name);
            if let Some(first) = first_occurance {
                return Err(Err::Failure(
                    vec![
                        (
                            name.marker(),
                            ParserErrorKind::DuplicateDefinition(name.to_string()),
                        ),
                        (first.name.marker(), ParserErrorKind::FirstOccurance),
                    ]
                    .into_iter()
                    .collect(),
                ));
            }
            fields.push(FieldValue {
                name,
                attributes,
                visibility,
                comments,
                field_type: ty,
                order: order as u64,
            });
        }

        Ok((s, Fields { fields, marker }))
    }
}

impl<I> ParserSerialize for Fields<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        let indents = ctx.create_indents();
        writeln!(f, "{{")?;

        let field_iter = self.iter();
        let mut first = true;
        for field_value in field_iter {
            if !first {
                writeln!(f, ",")?;
            } else {
                first = false;
            }
            let field_ctx = ctx.increment_indents(1);
            field_value.comments.compose(f, field_ctx)?;
            field_value.attributes.compose(f, field_ctx)?;
            field_value.visibility.compose(f, field_ctx)?;
            field_value.name.compose(f, field_ctx.set_indents(0))?;
            write!(f, ": ")?;
            field_value.field_type.compose(f, ctx.set_indents(0))?;
        }
        if !first {
            writeln!(f)?;
        }
        write!(f, "{indents}}}")?;
        Ok(())
    }
}

impl<I: Default> Default for Fields<I> {
    fn default() -> Self {
        Fields::new(Default::default(), Mark::null())
    }
}

impl<I> Marked<I> for Fields<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I: Hash> Hash for Fields<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for field in self.iter() {
            field.hash(state);
        }
    }
}

impl<I: PartialEq> PartialEq for Fields<I> {
    fn eq(&self, other: &Self) -> bool {
        let self_fields: Vec<_> = self.iter().collect();
        let other_fields: Vec<_> = other.iter().collect();

        if self_fields.len() != other_fields.len() {
            return false;
        }

        self_fields
            .into_iter()
            .zip(other_fields.into_iter())
            .all(|(l, r)| l == r)
    }
}

impl<I: PartialEq> Eq for Fields<I> {}

const FIELDS_DUMMY_LENGTH: usize = 10;

impl<I: Dummy<Faker>> Dummy<Faker> for Fields<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..FIELDS_DUMMY_LENGTH);

        let mut taken_names = HashSet::new();
        let mut fields = Vec::new();

        let mut i = 0;
        while i < len {
            let new_name: Ident<I> = Ident::dummy_with_rng(&Faker, rng);
            if &*new_name == "id" || taken_names.contains(&*new_name) {
                continue;
            }

            taken_names.insert(new_name.to_string());
            fields.push(new_name);
            i += 1;
        }

        Fields {
            fields: fields
                .into_iter()
                .enumerate()
                .map(|(i, name)| {
                    let mut value = FieldValue::dummy_with_rng(&Faker, rng);

                    value.name = Ident::new(name, Mark::dummy_with_rng(&Faker, rng));
                    value.order = i as u64;

                    value
                })
                .collect(),
            marker: Mark::dummy_with_rng(&Faker, rng),
        }
    }
}

pub(crate) struct FieldWithReferences(pub TypeReferenceMap);

impl<I: Dummy<Faker> + Clone> Dummy<FieldWithReferences> for Fields<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &FieldWithReferences,
        rng: &mut R,
    ) -> Self {
        let mut fields = Fields::dummy_with_rng(&Faker, rng);

        if config.0.is_empty() {
            fields.fields = Default::default();
            return fields;
        }

        for field_value in &mut fields.fields {
            config
                .0
                .pick_valid_reference_type(&mut field_value.field_type, rng);
        }

        fields
    }
}

compose_test! {fields_compose, Fields<I>}
