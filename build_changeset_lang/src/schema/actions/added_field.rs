use std::fmt::Display;

use build_script_lang::schema::FieldValue;
use build_script_lang::schema::Schema;
use build_script_lang::schema::Visibility;
use build_script_shared::InputType;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;

use fake::Dummy;
use nom::error::context;
use nom::sequence::*;
use nom::character::complete::*;

use crate::ChangeSetError;
use crate::ChangeSetResult;
use crate::FieldPath;

/// "+ \<ident\>.\<ident\>:\<type\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct AddedField<I> {
    pub(crate) comments: Comments,
    pub(crate) visibility: Visibility,
    pub(crate) field_path: FieldPath<I>,
    pub(crate) field_type: Types<I>,
}

impl<I> AddedField<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedField<O> 
    where
        F: Fn(I) -> O + Copy
    {
        AddedField {
            comments: self.comments,
            visibility: self.visibility,
            field_path: self.field_path.map(f),
            field_type: self.field_type.map(f)
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let (fields, field_name) = self.field_path.retrieve_fields(schema)
            .ok_or_else(||
                ChangeSetError::InvalidAction { 
                    action: format!("add field"), 
                    reason: format!("Failed to find type {}", self.field_path) 
                }
            )?;

        if fields.fields.contains_key(field_name) {
            return Err(ChangeSetError::InvalidAction { 
                action: format!("add field"), 
                reason: format!("field at {} already exists", self.field_path) 
            });
        }

        fields.fields.insert(field_name.clone(), FieldValue {
            visibility: self.visibility,
            comments: self.comments.get_doc_comments(),
            ty: self.field_type.clone()
        });

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedField<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, ((visibility, field_path), field_type))) = context(
            "Parsing AddedField",
            pair(
                Comments::parse,
                preceded(
                    ws(char('+')),
                    key_value(
                        pair(
                            Visibility::parse,
                            FieldPath::parse
                        ),
                        char(':'), 
                        Types::parse
                    )
                ),
            ),
        )(s)?;

        Ok((
            s,
            AddedField {
                comments,
                visibility,
                field_path,
                field_type,
            },
        ))
    }
}

impl<I> ParserSerialize for AddedField<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.comments.compose(f)?;
        write!(f, "+ ")?;
        self.visibility.compose(f)?;
        self.field_path.compose(f)?;
        write!(f, ": ")?;
        self.field_type.compose(f)?;
        Ok(())
    }
}

impl<I> Display for AddedField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
        .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{added_field_compose, AddedField<I>}