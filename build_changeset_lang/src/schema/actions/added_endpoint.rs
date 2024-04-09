use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use std::fmt::Display;

use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

/// "+ \<ident\>(\<ident\> => \<ident\>)"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct AddedEndpoint<I> {
    pub(crate) type_name: Ident<I>,
    pub(crate) endpoint: EndPoint<I>,
}

impl<I> AddedEndpoint<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedEndpoint<O>
    where
        F: Fn(I) -> O + Copy,
    {
        AddedEndpoint {
            type_name: self.type_name.map(f),
            endpoint: self.endpoint.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let edge = schema
            .get_type_mut(Some(SchemaStmType::Edge), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("add endpoint"),
                reason: format!("no edge type named {} exists", self.type_name),
            })?;

        if let SchemaStm::Edge(e) = edge {
            let key = (self.endpoint.source.clone(), self.endpoint.target.clone());
            if e.endpoints.contains_key(&key) {
                return Err(ChangeSetError::InvalidAction {
                    action: format!("add endpoint"),
                    reason: format!(
                        "{} already has {} => {}",
                        e.name, self.endpoint.source, self.endpoint.target
                    ),
                });
            }
            e.endpoints.insert(key, self.endpoint.clone());
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedEndpoint<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (type_name, endpoint)) = context(
            "Parsing AddedEndpoint",
            preceded(
                ws(char('+')),
                pair(Ident::ident, surrounded('(', EndPoint::parse, ')')),
            ),
        )(s)?;

        Ok((
            s,
            AddedEndpoint {
                type_name,
                endpoint,
            },
        ))
    }
}

impl<I> ParserSerialize for AddedEndpoint<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}+ ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, "( ")?;
        self.endpoint.compose(f, new_ctx)?;
        write!(f, " )")?;
        Ok(())
    }
}

impl<I> Display for AddedEndpoint<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {added_endpoint_compose, AddedEndpoint<I>}
