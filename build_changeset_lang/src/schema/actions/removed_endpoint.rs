use std::fmt::Display;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;
use nom::character::complete::*;
use build_script_lang::schema::*;
use crate::{ChangeSetError, ChangeSetResult};

/// "- \<ident\>(\<ident\> => \<ident\>)"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct RemovedEndpoint<I> {
    pub(crate) type_name: Ident<I>,
    pub(crate) endpoint: EndPoint<I>
}

impl<I> RemovedEndpoint<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> RemovedEndpoint<O> 
    where
        F: Fn(I) -> O + Copy
    {
        RemovedEndpoint {
            type_name: self.type_name.map(f),
            endpoint: self.endpoint.map(f)
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let edge = schema.get_type_mut(Some(SchemaStmType::Edge), &self.type_name).ok_or_else(|| {
            ChangeSetError::InvalidAction { 
                action: format!("remove endpoint"), 
                reason: format!("no edge type named {} exists", self.type_name) 
            }
        })?;

        if let SchemaStm::Edge(e) = edge {
            let is_removed = e.endpoints.remove(&(self.endpoint.source.clone(), self.endpoint.target.clone()));
            if is_removed.is_none() {
                return Err(ChangeSetError::InvalidAction { 
                    action: format!("remove endpoint"), 
                    reason: format!("{} is missing {} => {}", e.name, self.endpoint.source, self.endpoint.target)
                });
            }
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for RemovedEndpoint<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (type_name, endpoint)) = context(
            "Parsing RemovedEndpoint",
            preceded(
                ws(char('-')),
            
                pair(
                    Ident::ident, 
                    cut(surrounded('(', EndPoint::parse, ')'))
                ),
            ),
        )(s)?;
        Ok((
            s,
            RemovedEndpoint {
                type_name,
                endpoint,
            },
        ))
    }
}

impl<I> ParserSerialize for RemovedEndpoint<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        write!(f, "- ")?;
        self.type_name.compose(f)?;
        write!(f, "( ")?;
        self.endpoint.compose(f)?;
        write!(f, " )")?;
        Ok(())
    }
}

impl<I> Display for RemovedEndpoint<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{removed_endpoint_compose, RemovedEndpoint<I>}