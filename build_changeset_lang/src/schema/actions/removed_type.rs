use std::fmt::Display;

use build_script_lang::schema::SchemaStmType;
use build_script_shared::compose_test;
use build_script_shared::parsers::Ident;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

/// "- (node|edge|struct|enum) \<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct RemovedType<I> {
    pub type_type: SchemaStmType,
    pub type_name: Ident<I>,
}

impl<I> RemovedType<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> RemovedType<O>
    where
        F: Fn(I) -> O + Copy,
    {
        RemovedType {
            type_name: self.type_name.map(f),
            type_type: self.type_type,
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let stm = schema
            .get_type(Some(self.type_type), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("remove {} {}", self.type_type, self.type_name),
                reason: format!("no {} with that name exists", self.type_type),
            })?;

        let idx = schema
            .content
            .iter()
            .position(|ty| ty == stm)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("remove {} {}", self.type_type, self.type_name),
                reason: format!("no {} with that name exists", self.type_type),
            })?;
        schema.content.remove(idx);

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for RemovedType<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (ty, name)) = context(
            "Parsing RemovedType",
            preceded(ws(char('-')), pair(SchemaStmType::parse, ws(Ident::ident))),
        )(s)?;

        Ok((
            s,
            RemovedType {
                type_type: ty,
                type_name: name,
            },
        ))
    }
}

impl<I> ParserSerialize for RemovedType<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}- ")?;
        self.type_type.compose(f, new_ctx)?;
        write!(f, " ")?;
        self.type_name.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for RemovedType<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {removed_type_compose, RemovedType<I>}
