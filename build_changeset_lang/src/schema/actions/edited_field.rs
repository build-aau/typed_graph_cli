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
use crate::FieldPath;

/// "* \<ident\>.\<ident\>: \<type\> => \<type\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedField<I> {
    pub(crate) field_path: FieldPath<I>,
    pub(crate) old_visibility: Visibility,
    pub(crate) new_visibility: Visibility,
    pub(crate) old_type: Types<I>,
    pub(crate) new_type: Types<I>,
}

impl<I> EditedField<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedField<O> 
    where
        F: Fn(I) -> O + Copy
    {
        EditedField {
            field_path: self.field_path.map(f),
            old_visibility: self.old_visibility,
            new_visibility: self.new_visibility,
            old_type: self.old_type.map(f),
            new_type: self.new_type.map(f)
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let (fields, field_name) = self.field_path.retrieve_fields(schema)
            .ok_or_else(|| ChangeSetError::InvalidAction { 
                action: format!("edit field"), 
                reason: format!("Failed to find type {}", self.field_path) 
            })?;

        let field = fields.fields
            .get_mut(field_name)
            .ok_or_else(||ChangeSetError::InvalidAction { 
                action: format!("edit field"), 
                reason: format!("there exist no field with path {}", self.field_path) 
            })?;
            
        if &field.ty != &self.old_type {
            return Err(ChangeSetError::InvalidAction { 
                action: format!("edit field"), 
                reason: format!("old type of {} does not match, expected {} got {}", self.field_path, self.old_type, self.new_type) 
            });
        }

        if &field.visibility != &self.old_visibility {
            return Err(ChangeSetError::InvalidAction { 
                action: format!("edit field"), 
                reason: format!("old visibility of {} does not match, expected {} got {}", self.field_path, self.old_visibility, self.new_visibility) 
            });
        }

        field.ty = self.new_type.clone();
        field.visibility = self.new_visibility;

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedField<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (field_path, ((old_visibility, old_type), (new_visibility, new_type)))) = context(
            "Parsing EditedField",
            preceded(
                ws(char('*')),
                pair(
                    FieldPath::parse,
                    cut(preceded(
                        ws(char(':')),
                        key_value(
                            pair(
                                Visibility::parse,
                                Types::parse    
                            ), 
                            pair(char('='), char('>')), 
                            pair(
                                Visibility::parse,
                                Types::parse    
                            )
                        ),
                    )),
                ),
            ),
        )(s)?;

        Ok((
            s,
            EditedField {
                field_path,
                new_visibility,
                old_visibility,
                old_type,
                new_type,
            },
        ))
    }
}

impl<I> ParserSerialize for EditedField<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        write!(f, "* ")?;
        self.field_path.compose(f)?;
        write!(f, ": ")?;
        self.old_visibility.compose(f)?;
        self.old_type.compose(f)?;
        write!(f, " => ")?;
        self.new_visibility.compose(f)?;
        self.new_type.compose(f)?;
        Ok(())
    }
}

impl<I> Display for EditedField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{edited_field_compose, EditedField<I>}