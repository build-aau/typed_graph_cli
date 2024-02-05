use std::fmt::Display;

use build_script_shared::compose_test;
use build_script_shared::error::ParserSlimResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use crate::FieldPath;
use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;

/// "* \<ident\>.\<ident\>: \<type\> => \<type\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedField<I> {
    pub(crate) field_path: FieldPath<I>,
    pub(crate) comments: Comments,
    pub(crate) old_visibility: Visibility,
    pub(crate) new_visibility: Visibility,
    pub(crate) old_type: Types<I>,
    pub(crate) new_type: Types<I>,
    pub(crate) old_order: u64,
    pub(crate) new_order: u64,
}

impl<I> EditedField<I> {
    pub fn old_type(&self) -> &Types<I> {
        &self.old_type
    }

    pub fn new_type(&self) -> &Types<I> {
        &self.new_type
    }
    
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedField<O>
    where
        F: Fn(I) -> O + Copy,
    {
        EditedField {
            field_path: self.field_path.map(f),
            comments: self.comments,
            old_visibility: self.old_visibility,
            new_visibility: self.new_visibility,
            old_type: self.old_type.map(f),
            new_type: self.new_type.map(f),
            old_order: self.old_order,
            new_order: self.new_order

        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let named_fields = self.field_path.retrieve_field(schema)?;


        let named_key = self.field_path.get_field_name_res()?;
        let field = named_fields.get_field_mut(named_key.as_str())
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("edit field"),
                reason: format!("Failed to find named field at {}", self.field_path),
            })?;

        if &field.field_type != &self.old_type {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit field"),
                reason: format!(
                    "old type of {} does not match, expected {} got {}",
                    self.field_path, self.old_type, self.new_type
                ),
            });
        }

        if &field.visibility != &self.old_visibility {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit field"),
                reason: format!(
                    "old visibility of {} does not match, expected {} got {}",
                    self.field_path, self.old_visibility, self.new_visibility
                ),
            });
        }

        if &field.order != &self.old_order {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit field"),
                reason: format!(
                    "old order of {} does not match, expected {} got {}",
                    self.field_path, self.old_order, self.new_order
                ),
            });
        }

        field.comments.replace_doc_comments(&self.comments);
        field.field_type = self.new_type.clone();
        field.visibility = self.new_visibility;
        field.order = self.new_order;

        Ok(())
    }

    pub fn check_convertions(&self) -> ParserSlimResult<I, ()> 
    where
        I: Clone
    {
        self.old_type.check_convertion(&self.new_type)?;
        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedField<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, comments) = ws(Comments::parse)(s)?;

        let (s, (field_path, ((old_visibility, old_type, old_order), (new_visibility, new_type, new_order)))) =
            context(
                "Parsing EditedField",
                preceded(
                    ws(char('*')),
                    pair(
                        FieldPath::parse,
                        preceded(
                            ws(char(':')),
                            cut(key_value(
                                tuple((Visibility::parse, Types::parse, surrounded('(', u64, ')'))),
                                pair(char('='), char('>')),
                                tuple((Visibility::parse, Types::parse, surrounded('(', u64, ')'))),
                            )),
                        ),
                    ),
                ),
            )(s)?;

        Ok((
            s,
            EditedField {
                field_path,
                comments,
                new_visibility,
                old_visibility,
                old_type,
                new_type,
                old_order: old_order as u64,
                new_order: new_order as u64
            },
        ))
    }
}

impl<I> ParserSerialize for EditedField<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        self.comments.compose(f, ctx)?;
        write!(f, "{indents}* ")?;
        self.field_path.compose(f, new_ctx)?;
        write!(f, ": ")?;
        self.old_visibility.compose(f, new_ctx)?;
        self.old_type.compose(f, new_ctx)?;
        write!(f, "({})", self.old_order)?;
        write!(f, " => ")?;
        self.new_visibility.compose(f, new_ctx)?;
        self.new_type.compose(f, new_ctx)?;
        write!(f, "({})", self.new_order)?;
        Ok(())
    }
}

impl<I> Display for EditedField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {edited_field_compose, EditedField<I>}
