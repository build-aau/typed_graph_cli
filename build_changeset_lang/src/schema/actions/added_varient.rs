use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::InputType;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use fake::Dummy;
use nom::error::context;
use nom::sequence::*;
use nom::bytes::complete::*;
use nom::character::complete::*;

use crate::{ChangeSetError, ChangeSetResult};

/// "+ enum \<ident\>.\<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct AddedVarient<I> {
    pub(crate) comments: Comments,
    pub(crate) type_name: Ident<I>,
    pub(crate) varient_name: Ident<I>,
}

impl<I> AddedVarient<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedVarient<O> 
    where
        F: Fn(I) -> O + Copy
    {
        AddedVarient {
            comments: self.comments,
            type_name: self.type_name.map(f),
            varient_name: self.varient_name.map(f)
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let enum_stm = schema
            .get_type_mut(Some(SchemaStmType::Enum), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction { 
                action: format!("add varient"), 
                reason: format!("no enum type named {} exists", self.type_name) 
            })?;

        if let SchemaStm::Enum(e) = enum_stm {
            let name_collision = e.varients.contains_key(&self.varient_name);
            if name_collision {
                return Err(ChangeSetError::InvalidAction { 
                    action: format!("add varient"), 
                    reason: format!("varient {} already exists", self.varient_name) 
                });
            }
            e.varients.insert(self.varient_name.clone(), self.comments.get_doc_comments());
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedVarient<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, (type_name, varient_name))) = context(
            "Parsing AddedVarient",
            pair(
                Comments::parse,          
                preceded(
                    ws(char('+')),
                    preceded(
                        ws(tag("enum")), 
                        separated_pair(
                            Ident::ident, 
                            char('.'), 
                            Ident::ident
                        )
                    ),
                ),
            )
        )(s)?;

        Ok((
            s,
            AddedVarient { 
                comments,
                type_name, 
                varient_name 
            },
        ))
    }
}

impl<I> ParserSerialize for AddedVarient<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.comments.compose(f)?;
        write!(f, "+ enum ")?;
        self.type_name.compose(f)?;
        write!(f, ".")?;
        self.varient_name.compose(f)?;
        Ok(())
    }
}

impl<I> Display for AddedVarient<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{added_varient_compose, AddedVarient<I>}