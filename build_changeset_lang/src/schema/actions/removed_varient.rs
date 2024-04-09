use std::fmt::Display;

use build_script_lang::schema::SchemaStmType;
use build_script_shared::compose_test;
use build_script_shared::parsers::Ident;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

/// "- enum \<ident\>.\<ident\>""
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct RemovedVarient<I> {
    pub type_name: Ident<I>,
    pub varient_name: Ident<I>,
}

impl<I> RemovedVarient<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> RemovedVarient<O>
    where
        F: Fn(I) -> O + Copy,
    {
        RemovedVarient {
            type_name: self.type_name.map(f),
            varient_name: self.varient_name.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let enum_stm = schema
            .get_type_mut(Some(SchemaStmType::Enum), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("remove varient"),
                reason: format!("no enum type named {} exists", self.type_name),
            })?;

        if let SchemaStm::Enum(e) = enum_stm {
            let varient_idx = e.varient_position(&self.varient_name).ok_or_else(|| {
                ChangeSetError::InvalidAction {
                    action: format!("remove varient"),
                    reason: format!("no varient named {} exists", self.varient_name),
                }
            })?;

            e.varients.remove(varient_idx);
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for RemovedVarient<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (type_name, varient_name)) = context(
            "Parsing RemovedVarient",
            preceded(
                ws(char('-')),
                preceded(
                    tag("enum"),
                    ws(separated_pair(Ident::ident, char('.'), Ident::ident)),
                ),
            ),
        )(s)?;

        Ok((
            s,
            RemovedVarient {
                type_name,
                varient_name,
            },
        ))
    }
}

impl<I> ParserSerialize for RemovedVarient<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}- enum ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, ".")?;
        self.varient_name.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for RemovedVarient<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {removed_varient_compose, RemovedVarient<I>}
