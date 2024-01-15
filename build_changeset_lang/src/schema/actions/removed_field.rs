use std::fmt::Display;

use build_script_shared::InputType;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use fake::Dummy;

use crate::FieldPath;
use nom::error::context;
use nom::sequence::*;
use nom::character::complete::*;
use build_script_lang::schema::*;
use crate::{ChangeSetError, ChangeSetResult};

/// "- \<ident\>.\<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct RemovedField<I> {
    pub(crate) field_path: FieldPath<I>,
}

impl<I> RemovedField<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> RemovedField<O> 
    where
        F: Fn(I) -> O + Copy
    {
        RemovedField {
            field_path: self.field_path.map(f)
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let (fields, field_name) = self.field_path.retrieve_fields(schema)
            .ok_or_else(||
                ChangeSetError::InvalidAction { 
                    action: format!("remove field"), 
                    reason: format!("Failed to find type {}", self.field_path) 
                }
            )?;

        let removed = fields.fields.remove(field_name);
        if removed.is_none() {
            return Err(ChangeSetError::InvalidAction { 
                action: format!("remove field"), 
                reason: format!("field at {} did not exists", self.field_path) 
            });
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for RemovedField<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, field_path) = context(
            "Parsing RemovedField",
            preceded(
                ws(char('-')),
                FieldPath::parse
            ),
        )(s)?;

        Ok((
            s, 
            RemovedField { field_path }
        ))
    }
}

impl<I> ParserSerialize for RemovedField<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        write!(f, "- ")?;
        self.field_path.compose(f)?;
        Ok(())
    }
}

impl<I> Display for RemovedField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{removed_field_compose, RemovedField<I>}