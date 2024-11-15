use std::fmt::Display;

use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

/// "* \<ident\>.\<ident\>: \<type\> => \<type\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedVariant<I> {
    pub(crate) type_name: Ident<I>,
    pub(crate) varient_name: Ident<I>,
    pub(crate) comments: Comments,
    pub(crate) attributes: Attributes<I>,
}

impl<I> EditedVariant<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedVariant<O>
    where
        F: Fn(I) -> O + Copy,
    {
        EditedVariant {
            type_name: self.type_name.map(f),
            varient_name: self.varient_name.map(f),
            comments: self.comments,
            attributes: self.attributes.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let ty = schema
            .get_type_mut(Some(SchemaStmType::Enum), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("edit varient"),
                reason: format!("Failed to find enum type {}", self.type_name),
            })?;

        if let SchemaStm::Enum(e) = ty {
            let varient = e.get_varient_mut(&self.varient_name).ok_or_else(|| {
                ChangeSetError::InvalidAction {
                    action: format!("edit varient"),
                    reason: format!("{} is not an enum", self.type_name,),
                }
            })?;

            let varient_comments = varient.comments_mut();
            varient_comments.replace_doc_comments(&self.comments);

            let varient_attribtues = varient.attributes_mut();
            *varient_attribtues = self.attributes.clone();
        } else {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit varient"),
                reason: format!("{} is not an enum", self.type_name,),
            });
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedVariant<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, attributes, (type_name, varient_name))) = context(
            "Parsing EditedVariant",
            tuple((
                Comments::parse,
                Attributes::parse,
                preceded(
                    ws(char('*')),
                    separated_pair(Ident::ident, char('.'), Ident::ident),
                ),
            )),
        )(s)?;

        Ok((
            s,
            EditedVariant {
                type_name,
                varient_name,
                comments,
                attributes,
            },
        ))
    }
}

impl<I> ParserSerialize for EditedVariant<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        self.comments.compose(f, ctx)?;
        self.attributes.compose(f, ctx)?;
        write!(f, "{indents}* ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, ".")?;
        self.varient_name.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for EditedVariant<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {edited_varient_compose, EditedVariant<I>}
