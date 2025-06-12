use super::*;
use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::*;
use fake::Dummy;
use fake::Faker;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::*;
use nom::error::context;
use nom::multi::*;
use nom::sequence::*;
use nom::Err;
use serde::Deserialize;
use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub type DefaultSchema<'a> = Schema<InputMarkerRef<'a>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct Schema<I> {
    pub version: Ident<I>,
    pub handler: Option<Ident<I>>,
    #[serde(flatten)]
    pub comments: Comments,
    content: Vec<SchemaStm<I>>,
    #[serde(skip)]
    marker: Mark<I>,
}

impl<I> Schema<I> {
    pub fn new(comments: Comments, version: Ident<I>, handler: Option<Ident<I>>, content: Vec<SchemaStm<I>>, marker: Mark<I>) -> Self
    where
        I: Ord,
    {
        Schema {
            comments,
            version,
            handler,
            content: content.into_iter().collect(),
            marker,
        }
    }
}

impl<I> Schema<I> {
    pub fn nodes(&self) -> impl Iterator<Item = &NodeExp<I>>
    where
        I: Ord,
    {
        self.iter().filter_map(|stm| {
            if let SchemaStm::Node(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn edges(&self) -> impl Iterator<Item = &EdgeExp<I>>
    where
        I: Ord,
    {
        self.iter().filter_map(|stm| {
            if let SchemaStm::Edge(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn remove<S>(&mut self, stm: S) -> Option<SchemaStm<I>>
    where
        S: PartialEq<SchemaStm<I>>,
    {
        let idx = self.content.iter().position(|ty| &stm == ty)?;

        Some(self.content.remove(idx))
    }

    pub fn enums(&self) -> impl Iterator<Item = &EnumExp<I>>
    where
        I: Ord,
    {
        self.iter().filter_map(|stm| {
            if let SchemaStm::Enum(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn structs(&self) -> impl Iterator<Item = &StructExp<I>>
    where
        I: Ord,
    {
        self.iter().filter_map(|stm| {
            if let SchemaStm::Struct(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn imports(&self) -> impl Iterator<Item = &ImportExp<I>>
    where
        I: Ord,
    {
        self.iter().filter_map(|stm| {
            if let SchemaStm::Import(n) = stm {
                Some(n)
            } else {
                None
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &SchemaStm<I>>
    where
        I: Ord,
    {
        let mut content: Vec<_> = self.content.iter().collect();
        content.sort();
        content.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SchemaStm<I>>
    where
        I: Ord,
    {
        let mut content: Vec<_> = self.content.iter_mut().collect();
        content.sort();
        content.into_iter()
    }

    pub fn push(&mut self, stm: SchemaStm<I>) {
        self.content.push(stm);
    }

    pub fn extend(&mut self, other: Schema<I>) {
        self.content.extend(other.content);
    }

    /// Make sure that the parsed schema actually makes sense  
    /// This allows us to seperate the parser and type validation
    pub fn check_integrity(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        self.check_types()?;
        self.check_cycles()?;
        self.check_used()?;
        self.check_attributes()?;

        Ok(())
    }

    /// Check if attributes has the correct form
    fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        for stm in &self.content {
            match stm {
                SchemaStm::Edge(e) => e.check_attributes(),
                SchemaStm::Node(n) => n.check_attributes(),
                SchemaStm::Struct(s) => s.check_attributes(),
                SchemaStm::Enum(e) => e.check_attributes(),
                SchemaStm::Import(_) => Ok(()),
            }?;
        }

        Ok(())
    }

    /// Check if all type references are valid  
    fn check_types(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        let mut all_reference_types = HashSet::new();
        let mut node_reference_types = HashSet::new();
        let mut data_reference_types = HashMap::new();

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
                    data_reference_types.insert(type_name.clone(), Default::default());
                }
                SchemaStm::Struct(s) => {
                    data_reference_types.insert(type_name.clone(), s.generics.get_meta());
                }
                SchemaStm::Enum(e) => {
                    data_reference_types.insert(type_name.clone(), e.generics.get_meta());
                }
                SchemaStm::Import(_) => {
                    data_reference_types.insert(type_name.clone(), Default::default());
                }
                SchemaStm::Edge(_) => {
                    data_reference_types.insert(type_name.clone(), Default::default());
                }
            }
        }

        for stm in &self.content {
            match stm {
                SchemaStm::Node(n) => {
                    n.check_types(&data_reference_types, &node_reference_types)?
                }
                SchemaStm::Struct(s) => s.check_types(&data_reference_types)?,
                SchemaStm::Edge(e) => {
                    e.check_types(&data_reference_types, &node_reference_types)?
                }
                SchemaStm::Import(_) => (),
                SchemaStm::Enum(e) => e.check_types(&data_reference_types)?,
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

    /// Check if any type reference is causing a cycle to form  
    /// We do not allow cycles as it would require us to store all referenced type in a container such as Box
    fn check_cycles(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone + Default,
    {
        let mut dependency_graph = DependencyGraph::new();

        for stm in &self.content {
            dependency_graph.add_type(stm.get_type());
        }

        for stm in &self.content {
            match stm {
                SchemaStm::Node(n) => {
                    n.check_cycle(&mut dependency_graph)?;
                }
                SchemaStm::Struct(n) => {
                    n.check_cycle(&mut dependency_graph)?;
                }
                SchemaStm::Edge(e) => {
                    e.check_cycle(&mut dependency_graph)?;
                }
                SchemaStm::Enum(e) => {
                    e.check_cycle(&mut dependency_graph)?;
                }
                SchemaStm::Import(_) => (),
            }
        }

        Ok(())
    }

    /// Make sure that types that requires use are in use
    pub fn check_used(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        for stm in &self.content {
            match stm {
                SchemaStm::Enum(e) => e.check_used()?,
                SchemaStm::Struct(s) => s.check_used()?,
                SchemaStm::Node(_) | SchemaStm::Edge(_) | SchemaStm::Import(_) => (),
            }
        }

        Ok(())
    }

    pub fn strip_comments(&mut self) {
        for stm in &mut self.content {
            stm.strip_comments();
        }
    }

    pub fn get_type<T>(
        &self,
        stm_type: Option<SchemaStmType>,
        name: &T,
    ) -> Option<&SchemaStm<I>> 
    where 
        Ident<I>: PartialEq<T>
    {
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
            comments: self.comments,
            version: self.version.map(f),
            handler: self.handler.map(|h| h.map(f)),
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

    /// Parse a schema without doing performing and integrity check
    pub fn parse_no_check(requires_header: bool) -> impl Fn(I) -> ParserResult<I, Self>
    where
        I: InputType,
    {
        move |s: I| {
            let (s, header, comments, handler) = if requires_header {
                let (s, comments) = Comments::parse(s)?;
                let (s, (header, handler)) = context(
                    "Parsing Schema version",
                    surrounded('<', pair(ws(Ident::ident_full), opt(preceded(
                        tuple((ws(char(',')), tag("handler"), ws(char('=')))),
                        Ident::ident,
                    ))), '>'),
                )(s)?;

                if header.to_string().ends_with(".") {
                    return Err(Err::Failure(ParserError::new_at(
                        &header,
                        ParserErrorKind::Context("Cannot end on ."),
                    )));
                }

                (s, header, comments, handler)
            } else {
                (s, Ident::new_alone(""), Comments::new(Default::default()), None)
            };

            let (s, (content, marker)) = context(
                "Parsing Schema",
                marked(terminated(
                    many0(ws(SchemaStm::parse)),
                    context("Expected type declaration", eof),
                )),
            )(s)?;

            let schema = Schema {
                comments,
                version: header,
                handler,
                content,
                marker,
            };

            Ok((s, schema))
        }
    }
}

impl<I: Ord + Hash + Debug> Hash for Schema<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comments.hash(state);
        self.version.hash(state);
        self.handler.hash(state);
        let content: Vec<_> = self.iter().collect();
        content.hash(state);
    }
}

impl<I: Ord> PartialEq for Schema<I> {
    fn eq(&self, other: &Self) -> bool {
        let own_content: Vec<_> = self.iter().collect();
        let other_content: Vec<_> = other.iter().collect();

        self.comments.eq(&other.comments) && self.version.eq(&other.version) && own_content == other_content
    }
}

impl<I> Marked<I> for Schema<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I: InputType> ParserDeserialize<I> for Schema<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, schema) = Self::parse_no_check(true)(s)?;

        schema.check_integrity()?;

        Ok((s, schema))
    }
}

impl<I> ParserSerialize for Schema<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        if !self.version.is_empty() {
            self.comments.compose(f, ctx)?;
            write!(f, "<{}", self.version)?;
            if let Some(handler) = &self.handler {
                write!(f, ", handler=")?;
                handler.compose(f, ctx.set_indents(0))?;
            }
            writeln!(f, ">")?;
        }

        for stm in &self.content {
            stm.compose(f, ctx)?;
            writeln!(f, "")?;
        }

        Ok(())
    }
}

const SCHEMA_CONTENT_DUMMY_LENGTH: usize = 30;
const DUMMY_MAX_ORDER: usize = 10;
const DUMMY_MAX_GENERICS: usize = 5;

impl<I: Dummy<Faker> + Clone> Dummy<Faker> for Schema<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..SCHEMA_CONTENT_DUMMY_LENGTH);
        let mut content_type: Vec<_> = (0..len)
            .map(|_| {
                (
                    Ident::dummy_with_rng(config, rng),
                    rng.gen_range(0..DUMMY_MAX_ORDER),
                    rng.gen_range(0..DUMMY_MAX_GENERICS),
                    SchemaStmType::dummy_with_rng(config, rng),
                )
            })
            .collect();

        let mut all_types = HashSet::new();
        let mut node_type_names = HashSet::new();
        let mut ref_type_names: HashMap<_, HashMap<_, usize>> = HashMap::new();

        // Make sure that all the types do not overlap
        let mut i = 0;
        while let Some((ident, order, generic_count, ty)) = content_type.get_mut(i) {
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
                SchemaStmType::Struct | SchemaStmType::Enum => {
                    // Make sure the name is globally unique
                    if all_types.contains(&*name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    }

                    // Make sure the name is unique amongst reference types
                    let ref_type_names = ref_type_names.entry(*order).or_default();
                    if ref_type_names.contains_key(&name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    } else {
                        all_types.insert(name.to_string());
                        ref_type_names.insert(name, *generic_count);
                    }
                }
                SchemaStmType::Import => {
                    // Make sure the name is globally unique
                    if all_types.contains(&*name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    }

                    // Make sure the name is unique amongst reference types
                    let ref_type_names = ref_type_names.entry(*order).or_default();
                    if ref_type_names.contains_key(&name) {
                        // Update the name for next time we check
                        *ident = Ident::dummy_with_rng(&Faker, rng);
                        continue;
                    } else {
                        all_types.insert(name.to_string());
                        ref_type_names.insert(name, 0);
                    }
                }

                // Edges are never references
                SchemaStmType::Edge => (),
            }

            i += 1;
        }

        Schema {
            comments: Comments::dummy_with_rng(&Faker, rng),
            version: Ident::dummy_with_rng(config, rng),
            handler: if rng.gen_bool(0.5) {
                Some(Ident::dummy_with_rng(config, rng))
            } else {
                None
            },
            content: content_type
                .into_iter()
                .map(|(name, order, generic_count, ty)| {
                    SchemaStm::dummy_with_rng(
                        &SchemaStmOfType {
                            name,
                            ty,
                            generic_count,
                            node_types: node_type_names.clone(),
                            ref_types: TypeReferenceMap(
                                ref_type_names
                                    .iter()
                                    // Only allow references to values lower in the order than one self
                                    // This stops refference cycles from forming
                                    .filter(|(i, _)| &&order > i)
                                    .flat_map(|(_, types)| types.clone().into_iter())
                                    .collect(),
                            ),
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
fn schema_reorder_hash_test() {
    let import0: ImportExp<String> = ImportExp::new(
        Ident::new("Import0", Mark::dummy(&Faker)),
        Default::default(),
        Mark::dummy(&Faker),
    );

    let import1: ImportExp<String> = ImportExp::new(
        Ident::new("Import1", Mark::dummy(&Faker)),
        Default::default(),
        Mark::dummy(&Faker),
    );

    let version = Ident::dummy(&Faker);
    let mark: Mark<String> = Mark::dummy(&Faker);
    let comments = Comments::dummy(&Faker);

    let schema0 = Schema::new(
        comments.clone(),
        version.clone(),
        None,
        vec![
            SchemaStm::Import(import0.clone()),
            SchemaStm::Import(import1.clone()),
        ],
        mark.clone(),
    );

    let schema1 = Schema::new(
        comments,
        version,
        None,
        vec![SchemaStm::Import(import1), SchemaStm::Import(import0)],
        mark,
    );

    let hash0 = schema0.get_hash();
    let hash1 = schema1.get_hash();

    assert_eq!(hash0, hash1);
}

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

#[test]
fn cycle_proxy_test() {
    // This has the cycle A -> B -> C -> D -> E -> A
    let s0 = "
    <CycleProxy1>
    struct A {
        field: Proxy<B>
    };

    struct B {
        field: Proxy<C>
    };
    
    struct D {
        field: Proxy<E>
    };
    
    struct E {
        field: Proxy<A>
    };
    
    struct C {
        field: Proxy<D>
    };
    
    struct Proxy<K> {
        k: K
    };";

    // Has cycle A -> A
    let s1 = "
    <CycleProxy2>
    struct A {
        field: Proxy<A>
    };

    struct Proxy<K> {
        k: K
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
