use std::collections::BTreeMap;
use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub enum AddedTypeData<I> {
    Node,
    Struct,
    Edge {
        #[dummy(faker = "EndpointMap")]
        endpoints: BTreeMap<(Ident<I>, Ident<I>), EndPoint<I>>,
    },
    Enum,
    Import,
}

impl<I> AddedTypeData<I> {
    pub fn get_type(&self) -> SchemaStmType {
        match self {
            AddedTypeData::Node => SchemaStmType::Node,
            AddedTypeData::Struct => SchemaStmType::Struct,
            AddedTypeData::Enum => SchemaStmType::Enum,
            AddedTypeData::Import => SchemaStmType::Import,
            AddedTypeData::Edge { .. } => SchemaStmType::Edge,
        }
    }

    pub fn from_stm(stm: &SchemaStm<I>) -> AddedTypeData<I>
    where
        I: Clone,
    {
        match stm {
            SchemaStm::Node(_) => AddedTypeData::Node,
            SchemaStm::Struct(_) => AddedTypeData::Struct,
            SchemaStm::Enum(_) => AddedTypeData::Enum,
            SchemaStm::Import(_) => AddedTypeData::Import,
            SchemaStm::Edge(e) => AddedTypeData::Edge {
                endpoints: e.endpoints.clone(),
            },
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedTypeData<O>
    where
        F: Fn(I) -> O + Copy,
    {
        match self {
            AddedTypeData::Node => AddedTypeData::Node,
            AddedTypeData::Struct => AddedTypeData::Struct,
            AddedTypeData::Enum => AddedTypeData::Enum,
            AddedTypeData::Import => AddedTypeData::Import,
            AddedTypeData::Edge { endpoints } => AddedTypeData::Edge {
                endpoints: endpoints
                    .into_iter()
                    .map(|((source, target), endpoint)| {
                        ((source.map(f), target.map(f)), endpoint.map(f))
                    })
                    .collect(),
            },
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedTypeData<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        context(
            "Parsing AddedTypeData",
            alt((
                value(AddedTypeData::Node, tag("node")),
                value(AddedTypeData::Struct, tag("struct")),
                value(AddedTypeData::Enum, tag("enum")),
                value(AddedTypeData::Import, tag("import")),
                map(
                    pair(ws(tag("edge")), EdgeExp::parse_endpoints),
                    |(_, endpoints)| AddedTypeData::Edge { endpoints },
                ),
            )),
        )(s)
    }
}

impl<I> ParserSerialize for AddedTypeData<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();

        match self {
            AddedTypeData::Node => write!(f, "{indents}node")?,
            AddedTypeData::Struct => write!(f, "{indents}struct")?,
            AddedTypeData::Enum => write!(f, "{indents}enum")?,
            AddedTypeData::Import => write!(f, "{indents}import")?,
            AddedTypeData::Edge { endpoints } => {
                let mut first = true;
                write!(f, "{indents}edge ( ")?;
                for endpoint in endpoints.values() {
                    if !first {
                        writeln!(f, ",")?;
                    } else {
                        writeln!(f, "")?;
                        first = false;
                    }
                    endpoint.compose(f, ctx.increment_indents(1))?;
                }
                if !first {
                    writeln!(f, "{indents}")?;
                }
                write!(f, " )")?;
            }
        }

        Ok(())
    }
}

impl<I> Display for AddedTypeData<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {added_type_data_compose, AddedTypeData<I>}
