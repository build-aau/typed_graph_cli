use super::*;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::*;
use fake::Dummy;
use fake::Faker;
use nom::combinator::*;
use nom::error::context;
use nom::multi::*;
use nom::sequence::*;
use nom::Err;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub type DefaultSchema<'a> = Schema<InputMarkerRef<'a>>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Schema<I> {
    pub version: Ident<I>,
    pub content: Vec<SchemaStm<I>>,
    marker: Mark<I>,
}

impl<I> Schema<I> {
    pub fn new(version: Ident<I>, content: Vec<SchemaStm<I>>, marker: Mark<I>) -> Self
    where
        I: Ord,
    {
        Schema {
            version,
            content: content.into_iter().collect(),
            marker,
        }
    }
}

impl<I> Schema<I> {
    pub fn check_integrity(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        self.check_attributes()?;
        self.check_types()?;
        self.check_cycles()?;

        Ok(())
    }

    pub fn nodes(&self) -> impl Iterator<Item = &NodeExp<I>> {
        self.content.iter().filter_map(|stm| {
            if let SchemaStm::Node(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn edges(&self) -> impl Iterator<Item = &EdgeExp<I>> {
        self.content.iter().filter_map(|stm| {
            if let SchemaStm::Edge(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn enums(&self) -> impl Iterator<Item = &EnumExp<I>> {
        self.content.iter().filter_map(|stm| {
            if let SchemaStm::Enum(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn structs(&self) -> impl Iterator<Item = &StructExp<I>> {
        self.content.iter().filter_map(|stm| {
            if let SchemaStm::Struct(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        for stm in &self.content {
            match stm {
                SchemaStm::Edge(e) => e.check_attributes(),
                SchemaStm::Node(n) => n.check_attributes(),
                SchemaStm::Struct(_) => Ok(()),
                SchemaStm::Enum(_) => Ok(()),
                SchemaStm::Import(_) => Ok(()),
            }?;
        }

        Ok(())
    }

    fn check_types(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        let mut all_reference_types = HashSet::new();
        let mut node_reference_types = HashSet::new();
        let mut data_reference_types = HashSet::new();

        for stm in &self.content {
            let type_name = stm.get_type();

            let first = all_reference_types
                .iter()
                .find(|v| type_name.cmp(v).is_eq())
                .cloned();

            if !all_reference_types.insert(type_name.clone()) {
                let first = first.unwrap();
                return Err(Err::Failure(
                    vec![
                        (
                            type_name.marker(),
                            ParserErrorKind::DuplicateDefinition(type_name.to_string()),
                        ),
                        (first.marker(), ParserErrorKind::FirstOccurance),
                    ]
                    .into_iter()
                    .collect(),
                ));
            }

            match stm {
                SchemaStm::Node(_) => {
                    node_reference_types.insert(type_name.clone());
                }
                SchemaStm::Struct(_) => {
                    data_reference_types.insert(type_name.clone());
                }
                SchemaStm::Enum(_) => {
                    data_reference_types.insert(type_name.clone());
                }
                SchemaStm::Import(_) => {
                    data_reference_types.insert(type_name.clone());
                }
                SchemaStm::Edge(_) => (),
            }
        }

        for stm in &self.content {
            match stm {
                SchemaStm::Node(n) => {
                    n.check_types(&node_reference_types)?;
                    n.fields.check_types(&data_reference_types)?
                }
                SchemaStm::Struct(n) => n.fields.check_types(&data_reference_types)?,
                SchemaStm::Edge(e) => {
                    e.check_types(&data_reference_types, &node_reference_types)?;
                    e.fields.check_types(&data_reference_types)?
                }
                SchemaStm::Import(_) => (),
                SchemaStm::Enum(_) => (),
            }
        }

        let mut node_name_type: Option<(&Types<I>, String)> = None;
        let mut edge_name_type: Option<(&Types<I>, String)> = None;
        for stm in &self.content {
            match stm {
                SchemaStm::Node(n) => {
                    n.fields.check_field_type("name", &mut node_name_type)?;
                }
                SchemaStm::Edge(e) => {
                    e.fields.check_field_type("name", &mut edge_name_type)?;
                }
                SchemaStm::Struct(_) => (),
                SchemaStm::Import(_) => (),
                SchemaStm::Enum(_) => (),
            }
        }

        Ok(())
    }

    fn check_cycles(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        let mut dependency_graph: HashMap<&Ident<I>, Vec<&Ident<I>>> = HashMap::new();

        for stm in &self.content {
            dependency_graph.insert(stm.get_type(), Vec::new());
        }

        for stm in &self.content {
            match stm {
                SchemaStm::Node(n) => {
                    n.fields.check_cycle(&n.name, &mut dependency_graph)?;
                }
                SchemaStm::Struct(n) => {
                    n.fields.check_cycle(&n.name, &mut dependency_graph)?;
                }
                SchemaStm::Edge(e) => {
                    e.fields.check_cycle(&e.name, &mut dependency_graph)?;
                }
                SchemaStm::Enum(_) => (),
                SchemaStm::Import(_) => (),
            }
        }

        Ok(())
    }

    pub fn get_type(
        &self,
        stm_type: Option<SchemaStmType>,
        name: &Ident<I>,
    ) -> Option<&SchemaStm<I>> {
        for stm in &self.content {
            let is_allowed_type = stm_type
                .map(|ty| ty == stm.get_schema_type())
                .unwrap_or_else(|| true);
            let name_collision = is_allowed_type && stm.get_type() == name;

            if name_collision {
                return Some(stm);
            }
        }

        None
    }

    pub fn get_type_mut(
        &mut self,
        stm_type: Option<SchemaStmType>,
        name: &Ident<I>,
    ) -> Option<&mut SchemaStm<I>> {
        for stm in &mut self.content {
            let is_allowed_type = stm_type
                .map(|ty| ty == stm.get_schema_type())
                .unwrap_or_else(|| true);
            let name_collision = is_allowed_type && stm.get_type() == name;
            if name_collision {
                return Some(stm);
            }
        }

        None
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Schema<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Schema {
            version: self.version.map(f),
            content: self.content.into_iter().map(|stm| stm.map(f)).collect(),
            marker: self.marker.map(f),
        }
    }

    pub fn get_hash(&self) -> u64
    where
        I: Hash + Ord + Debug,
    {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl<I: Ord + Hash + Debug> Hash for Schema<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.version.hash(state);
        let content: BTreeSet<_> = self.content.iter().collect();
        content.hash(state);
    }
}

impl<I> Marked<I> for Schema<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I: InputType> ParserDeserialize<I> for Schema<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, version) = context(
            "Parsing Schema version",
            surrounded('<', ws(Ident::ident_full), '>'),
        )(s)?;
        let (s, (content, marker)) = context(
            "Parsing Schema",
            marked(terminated(
                many0(ws(SchemaStm::parse)),
                context("Expected type declaration", eof),
            )),
        )(s)?;

        if version.to_string().ends_with(".") {
            return Err(Err::Failure(ParserError::new_at(
                &version,
                ParserErrorKind::Context("Cannot end on ."),
            )));
        }

        let schema = Schema {
            version: version,
            content,
            marker,
        };

        schema.check_integrity()?;

        Ok((s, schema))
    }
}

impl<I> ParserSerialize for Schema<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        writeln!(f, "<{}>", self.version)?;

        for stm in &self.content {
            stm.compose(f)?;
            writeln!(f, "")?;
        }

        Ok(())
    }
}

const SCHEMA_CONTENT_DUMMY_LENGTH: usize = 30;
const DUMMY_MAX_ORDER: usize = 10;

impl<I: Dummy<Faker> + Clone> Dummy<Faker> for Schema<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..SCHEMA_CONTENT_DUMMY_LENGTH);
        let mut content_type: Vec<_> = (0..len)
            .map(|_| {
                (
                    Ident::dummy_with_rng(config, rng),
                    rng.gen_range(0..DUMMY_MAX_ORDER),
                    SchemaStmType::dummy_with_rng(config, rng),
                )
            })
            .collect();

        let mut all_types = HashSet::new();
        let mut node_type_names = HashSet::new();
        let mut ref_type_names: HashMap<_, HashSet<_>> = HashMap::new();

        // Make sure that all the types do not overlap
        let mut i = 0;
        while let Some((ident, order, ty)) = content_type.get_mut(i) {
            let name = ident.to_string();
            match ty {
                SchemaStmType::Node => {
                    // Make sure the name is globally unique
                    if all_types.contains(&*name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    }

                    // Make sure the name is unique amongst nodes
                    if node_type_names.contains(&name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    } else {
                        all_types.insert(name.to_string());
                        node_type_names.insert(name);
                    }
                }
                SchemaStmType::Struct | SchemaStmType::Enum | SchemaStmType::Import => {
                    // Make sure the name is globally unique
                    if all_types.contains(&*name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    }

                    // Make sure the name is unique amongst reference types
                    let ref_type_names = ref_type_names.entry(*order).or_default();
                    if ref_type_names.contains(&name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    } else {
                        all_types.insert(name.to_string());
                        ref_type_names.insert(name);
                    }
                }

                // Edges are never references
                SchemaStmType::Edge => (),
            }

            i += 1;
        }

        Schema {
            version: Ident::dummy_with_rng(config, rng),
            content: content_type
                .into_iter()
                .map(|(name, order, ty)| {
                    SchemaStm::dummy_with_rng(
                        &SchemaStmOfType {
                            name,
                            ty,
                            node_types: node_type_names.clone(),
                            ref_types: ref_type_names
                                .iter()
                                // Only allow references to values lower in the order than one self
                                // This stops refference cycles from forming
                                .filter(|(i, _)| &&order > i)
                                .flat_map(|(_, types)| types.iter())
                                .cloned()
                                .collect(),
                        },
                        rng,
                    )
                })
                .collect(),
            marker: Mark::dummy_with_rng(config, rng),
        }
    }
}

compose_test! {schema_compose, Schema<I>}

#[test]
fn cycle_test() {
    // This has the cycle A -> B -> C -> D -> E -> A
    let s0 = "
    <Cycle1>
    struct A {
        field: B
    };

    struct B {
        field: C
    };
    
    struct D {
        field: E
    };
    
    struct E {
        field: A
    };
    
    struct C {
        field: D
    };";

    // Has cycle A -> A
    let s1 = "
    <Cycle2>
    struct A {
        field: A
    };";

    assert_eq!(
        Schema::parse(s0),
        ParserResult::<_, _>::Err(Err::Failure(ParserError::new(
            "D",
            ParserErrorKind::CyclicReference
        )))
    );
    assert_eq!(
        Schema::parse(s1),
        ParserResult::<_, _>::Err(Err::Failure(ParserError::new(
            "A",
            ParserErrorKind::CyclicReference
        )))
    );
}
