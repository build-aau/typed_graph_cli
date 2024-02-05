use std::collections::HashMap;
use std::collections::HashSet;

use super::*;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::*;
use fake::Dummy;
use fake::Faker;
use nom::branch::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(PartialEq, Eq, Debug, Hash, Clone, PartialOrd, Ord, Dummy, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
#[serde(tag = "type")]
pub enum SchemaStm<I> {
    Node(NodeExp<I>),
    Edge(EdgeExp<I>),
    Enum(EnumExp<I>),
    Struct(StructExp<I>),
    Import(ImportExp<I>),
}

impl<I> SchemaStm<I> {
    pub fn get_type(&self) -> &Ident<I> {
        match self {
            SchemaStm::Node(n) => &n.name,
            SchemaStm::Edge(n) => &n.name,
            SchemaStm::Enum(n) => &n.name,
            SchemaStm::Struct(n) => &n.name,
            SchemaStm::Import(n) => &n.name,
        }
    }

    pub fn get_schema_type(&self) -> SchemaStmType {
        match self {
            SchemaStm::Node(_n) => SchemaStmType::Node,
            SchemaStm::Edge(_n) => SchemaStmType::Edge,
            SchemaStm::Enum(_n) => SchemaStmType::Enum,
            SchemaStm::Struct(_n) => SchemaStmType::Struct,
            SchemaStm::Import(_n) => SchemaStmType::Import,
        }
    }

    pub fn get_fields(&self) -> Option<&Fields<I>> {
        match self {
            SchemaStm::Node(n) => Some(&n.fields),
            SchemaStm::Edge(n) => Some(&n.fields),
            SchemaStm::Struct(n) => Some(&n.fields),
            SchemaStm::Enum(_) => None,
            SchemaStm::Import(_) => None,
        }
    }

    pub fn get_fields_mut(&mut self) -> Option<&mut Fields<I>> {
        match self {
            SchemaStm::Node(n) => Some(&mut n.fields),
            SchemaStm::Edge(n) => Some(&mut n.fields),
            SchemaStm::Struct(n) => Some(&mut n.fields),
            SchemaStm::Enum(_) => None,
            SchemaStm::Import(_) => None,
        }
    }

    pub fn get_comments(&self) -> &Comments {
        match self {
            SchemaStm::Node(n) => &n.comments,
            SchemaStm::Edge(n) => &n.comments,
            SchemaStm::Struct(n) => &n.comments,
            SchemaStm::Enum(n) => &n.comments,
            SchemaStm::Import(n) => &n.comments,
        }
    }

    pub fn get_attributes(&self) -> Option<&Attributes<I>> {
        match self {
            SchemaStm::Node(_) => None,
            SchemaStm::Edge(n) => Some(&n.attributes),
            SchemaStm::Struct(_) => None,
            SchemaStm::Enum(_) => None,
            SchemaStm::Import(_) => None,
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> SchemaStm<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            SchemaStm::Node(n) => SchemaStm::Node(n.map(f)),
            SchemaStm::Edge(n) => SchemaStm::Edge(n.map(f)),
            SchemaStm::Enum(n) => SchemaStm::Enum(n.map(f)),
            SchemaStm::Struct(n) => SchemaStm::Struct(n.map(f)),
            SchemaStm::Import(n) => SchemaStm::Import(n.map(f)),
        }
    }

    pub fn strip_comments(&mut self) {
        match self {
            SchemaStm::Node(n) => n.strip_comments(),
            SchemaStm::Edge(n) => n.strip_comments(),
            SchemaStm::Enum(n) => n.strip_comments(),
            SchemaStm::Struct(n) => n.strip_comments(),
            SchemaStm::Import(n) => n.strip_comments(),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for SchemaStm<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        context(
            "Parsing Schema Statement",
            terminated(
                alt((
                    map(NodeExp::parse, SchemaStm::Node),
                    map(EdgeExp::parse, SchemaStm::Edge),
                    map(EnumExp::parse, SchemaStm::Enum),
                    map(StructExp::parse, SchemaStm::Struct),
                    map(ImportExp::parse, SchemaStm::Import),
                    fail,
                )),
                cut(char(';')),
            )
        )(s)
    }
}

impl<I> ParserSerialize for SchemaStm<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        match self {
            SchemaStm::Node(n) => n.compose(f, ctx),
            SchemaStm::Edge(n) => n.compose(f, ctx),
            SchemaStm::Enum(n) => n.compose(f, ctx),
            SchemaStm::Struct(n) => n.compose(f, ctx),
            SchemaStm::Import(n) => n.compose(f, ctx),
        }?;
        write!(f, ";")?;
        Ok(())
    }
}

impl<I> Marked<I> for SchemaStm<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            SchemaStm::Node(n) => n.marker(),
            SchemaStm::Edge(n) => n.marker(),
            SchemaStm::Enum(n) => n.marker(),
            SchemaStm::Struct(n) => n.marker(),
            SchemaStm::Import(n) => n.marker(),
        }
    }
}

pub struct SchemaStmOfType<I> {
    pub name: Ident<I>,
    pub ty: SchemaStmType,
    pub generic_count: usize,
    pub node_types: HashSet<String>,
    pub ref_types: TypeReferenceMap,
}

impl<I: Dummy<Faker> + Clone> Dummy<SchemaStmOfType<I>> for SchemaStm<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &SchemaStmOfType<I>,
        rng: &mut R,
    ) -> Self {
        match &config.ty {
            SchemaStmType::Node => SchemaStm::Node(NodeExp::dummy_with_rng(
                &NodeExpOfType {
                    name: config.name.clone(),
                    ref_types: config.ref_types.clone(),
                    node_types: config.node_types.clone(),
                },
                rng,
            )),
            SchemaStmType::Edge => SchemaStm::Edge(EdgeExp::dummy_with_rng(
                &EdgeExpOfType {
                    name: config.name.clone(),
                    node_types: config.node_types.clone(),
                    ref_types: config.ref_types.clone(),
                },
                rng,
            )),
            SchemaStmType::Struct => SchemaStm::Struct(StructExp::dummy_with_rng(
                &StructExpOfType {
                    name: config.name.clone(),
                    generic_count: config.generic_count,
                    ref_types: config.ref_types.clone(),
                },
                rng,
            )),
            SchemaStmType::Enum => SchemaStm::Enum(EnumExp::dummy_with_rng(
                &EnumExpOfType {
                    name: config.name.clone(),
                    generic_count: config.generic_count,
                    ref_types: config.ref_types.clone(),
                },
                rng,
            )),
            SchemaStmType::Import => SchemaStm::Import(ImportExp::dummy_with_rng(
                &ImportExpOfType {
                    name: config.name.clone(),
                },
                rng,
            )),
        }
    }
}

compose_test! {schema_stm_compose, SchemaStm<I>}
