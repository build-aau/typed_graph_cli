use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::*;
use fake::Dummy;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::error::context;
use std::fmt::Display;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Dummy)]
pub enum SchemaStmType {
    Node,
    Edge,
    Enum,
    Struct,
    Import,
}

impl<I: InputType> ParserDeserialize<I> for SchemaStmType {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, schema) = context(
            "Parsing Schema Statement Type",
            alt((
                value(SchemaStmType::Node, tag("node")),
                value(SchemaStmType::Edge, tag("edge")),
                value(SchemaStmType::Enum, tag("enum")),
                value(SchemaStmType::Struct, tag("struct")),
                value(SchemaStmType::Import, tag("import")),
            )),
        )(s)?;

        Ok((s, schema))
    }
}

impl ParserSerialize for SchemaStmType {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        match self {
            SchemaStmType::Node => write!(f, "node"),
            SchemaStmType::Edge => write!(f, "edge"),
            SchemaStmType::Enum => write!(f, "enum"),
            SchemaStmType::Struct => write!(f, "struct"),
            SchemaStmType::Import => write!(f, "import"),
        }?;
        Ok(())
    }
}

impl Display for SchemaStmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {schema_type_compose, SchemaStmType}
