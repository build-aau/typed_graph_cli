use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Faker;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::pair;
use nom::Err;
use rand::seq::IteratorRandom;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use super::Visibility;

#[derive(PartialEq, Eq, Debug, Hash, Clone, PartialOrd, Ord)]
pub struct Fields<I> {
    pub fields: BTreeMap<Ident<I>, FieldValue<I>>,
    marker: Mark<I>,
}

#[derive(Debug, Hash, Clone, PartialOrd, Ord, Dummy, PartialEq, Eq)]
pub struct FieldValue<I> {
    pub visibility: Visibility,
    pub comments: Comments,
    pub ty: Types<I>,
}

impl<I> Fields<I> {
    pub fn new(fields: BTreeMap<Ident<I>, FieldValue<I>>, marker: Mark<I>) -> Fields<I> {
        Fields { fields, marker }
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn has_field<'a, S: PartialEq<&'a str>>(&'a self, field: S) -> bool {
        self.fields.iter().any(|(name, _)| field == name.as_ref())
    }

    pub fn get_field<'a, S: PartialEq<&'a str>>(
        &'a self,
        field: S,
    ) -> Option<(&Ident<I>, &FieldValue<I>)> {
        self.fields.iter().find(|(name, _)| field == name.as_ref())
    }

    pub fn check_types(&self, all_reference_types: &HashSet<Ident<I>>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for (_, field_value) in &self.fields {
            if let Err(err_ty) = field_value.ty.is_valid(all_reference_types) {
                return Err(Err::Failure(ParserError::new_at(
                    err_ty,
                    ParserErrorKind::UnknownReference(err_ty.to_string()),
                )));
            }
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
        if let Some((field_name, field_value)) = self.get_field(field) {
            if let Some((ty_origin, ty)) = &id_type {
                if ty != &field_value.ty.to_string() {
                    return Err(Err::Failure(
                        vec![
                            (
                                field_name.marker(),
                                ParserErrorKind::UnexpectedFieldType(
                                    field.to_string(),
                                    field_value.ty.to_string(),
                                ),
                            ),
                            (ty_origin.marker(), ParserErrorKind::FirstOccurance),
                        ]
                        .into_iter()
                        .collect(),
                    ));
                }
            } else {
                *id_type = Some((&field_value.ty, field_value.ty.to_string()));
            }
        }

        Ok(())
    }

    pub fn check_cycle<'a>(
        &'a self,
        node_name: &'a Ident<I>,
        dependency_graph: &mut HashMap<&Ident<I>, Vec<&'a Ident<I>>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for (_, field_value) in &self.fields {
            if let Types::Reference(ty_ref) = &field_value.ty {
                if dependency_graph.contains_key(ty_ref) {
                    let outgoing = dependency_graph.get_mut(node_name).unwrap();
                    outgoing.push(ty_ref);

                    // Check if the refference creates a cyclic dependency
                    if Fields::is_cyclic_directed_graph(&dependency_graph) {
                        return Err(Err::Failure(ParserError::new_at(
                            &field_value.ty,
                            ParserErrorKind::CyclicReference,
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Cycle detection in a directed graph using DFS
    fn is_cyclic_directed_graph(graph: &HashMap<&Ident<I>, Vec<&Ident<I>>>) -> bool {
        // set is used to mark visited vertices
        let mut visited = HashSet::new();
        // set is used to keep track the ancestor vertices in recursive stack.
        let mut ancestors = HashSet::new();

        // call recur for all vertices
        for u in graph.keys() {
            // Don't recur for u if it is already visited
            if !visited.contains(u)
                && Fields::is_cyclic_recur(&graph, u, &mut visited, &mut ancestors)
            {
                return true;
            }
        }

        false
    }

    fn is_cyclic_recur<'a>(
        graph: &HashMap<&'a Ident<I>, Vec<&'a Ident<I>>>,
        current_vertex: &'a Ident<I>,
        visited: &mut HashSet<&'a Ident<I>>,
        ancestors: &mut HashSet<&'a Ident<I>>,
    ) -> bool {
        // mark it visited
        visited.insert(current_vertex);
        // add it to ancestor vertices
        ancestors.insert(current_vertex);

        // Recur for all the vertices adjacent to current_vertex
        for v in &graph[current_vertex] {
            // If the vertex is not visited then recurse on it
            if !visited.contains(v) {
                if Fields::is_cyclic_recur(graph, v, visited, ancestors) {
                    return true;
                }
            } else if ancestors.contains(v) {
                // found a back edge, so there is a cycle
                return true;
            }
        }

        // remove from the ancestor vertices
        ancestors.remove(current_vertex);

        false
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
                .map(|(k, v)| {
                    (
                        k.map(f),
                        FieldValue {
                            visibility: v.visibility,
                            comments: v.comments,
                            ty: v.ty.map(f),
                        },
                    )
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
                    pair(
                        Comments::parse,
                        key_value(pair(Visibility::parse, Ident::ident), char(':'), Types::parse),
                    ),
                    ',',
                ),
                '}',
            )),
        )(s)?;

        // Populate the list of fields
        let mut fields: BTreeMap<Ident<I>, FieldValue<I>> = BTreeMap::new();
        for (comments, ((visibility, k), ty)) in fields_raw {
            if fields.contains_key(&k) {
                let field = fields.keys().find(|f| f == &&k).unwrap();
                return Err(Err::Failure(
                    vec![
                        (
                            k.marker(),
                            ParserErrorKind::DuplicateDefinition(k.to_string()),
                        ),
                        (field.marker(), ParserErrorKind::FirstOccurance),
                    ]
                    .into_iter()
                    .collect(),
                ));
            }
            fields.insert(k, FieldValue { visibility, comments, ty });
        }

        Ok((s, Fields { fields, marker }))
    }
}

impl<I> ParserSerialize for Fields<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        writeln!(f, "{{")?;

        let field_iter = self.fields.iter().enumerate();
        let mut first = true;
        for (_, (field_name, field_value)) in field_iter {
            if !first {
                writeln!(f, ",")?;
            } else {
                first = false;
            }
            field_value.comments.compose(f)?;
            field_value.visibility.compose(f)?;
            field_name.compose(f)?;
            write!(f, ": ")?;
            field_value.ty.compose(f)?;
        }

        write!(f, "}}")?;
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

const FIELDS_DUMMY_LENGTH: usize = 10;

impl<I: Dummy<Faker>> Dummy<Faker> for Fields<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..FIELDS_DUMMY_LENGTH);

        let mut taken_names = HashSet::new();
        let mut fields = Vec::new();

        let mut i = 0;
        while i < len {
            let new_name: Ident<I> = Ident::dummy_with_rng(&Faker, rng);
            if *new_name == "id" || taken_names.contains(&*new_name) {
                continue;
            }

            taken_names.insert(new_name.to_string());
            fields.push(new_name);
            i += 1;
        }

        Fields {
            fields: fields
                .into_iter()
                .map(|name| {
                    (
                        Ident::new(name, Mark::dummy_with_rng(&Faker, rng)),
                        FieldValue::dummy_with_rng(&Faker, rng),
                    )
                })
                .collect(),
            marker: Mark::dummy_with_rng(&Faker, rng),
        }
    }
}

fn fix_type_references<I: Dummy<Faker>, R: rand::prelude::Rng + ?Sized>(ty: &mut Types<I>, config: &FieldWithReferences, rng: &mut R) {
    match ty {
        Types::String(_) => (),
        Types::Bool(_) => (),
        Types::F64(_) => (),
        Types::F32(_) => (),
        Types::Usize(_) => (),
        Types::U64(_) => (),
        Types::U32(_) => (),
        Types::U16(_) => (),
        Types::U8(_) => (),
        Types::Isize(_) => (),
        Types::I64(_) => (),
        Types::I32(_) => (),
        Types::I16(_) => (),
        Types::I8(_) => (),
        Types::Option(ty, _) => fix_type_references(ty, config, rng),
        Types::List(ty, _) => fix_type_references(ty, config, rng),
        Types::Map(kty, vty, _) =>  {
            fix_type_references(kty, config, rng);
            fix_type_references(vty, config, rng);
        },
        Types::Reference(r) => {
            *r = Ident::new(config.0.iter().choose(rng).unwrap(), Mark::dummy_with_rng(&Faker, rng));
        }
    }
}

pub(crate) struct FieldWithReferences(pub HashSet<String>);
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

        let keys: Vec<_> = fields.fields.keys().cloned().collect();
        for key in keys {
            let field_value = fields.fields.get_mut(&key).unwrap();

            fix_type_references(&mut field_value.ty, config, rng);
        }

        fields
    }
}

compose_test! {fields_compose, Fields<I>}
