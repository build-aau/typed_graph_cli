use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::character::complete::char;
use nom::error::context;
use nom::sequence::*;

use crate::{ChangeSetError, ChangeSetResult};

#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedGenerics<I> {
    pub type_name: Ident<I>,
    pub old_generics: Generics<I>,
    pub new_generics: Generics<I>,
}

impl<I> EditedGenerics<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedGenerics<O>
    where
        F: Fn(I) -> O + Copy,
    {
        EditedGenerics {
            type_name: self.type_name.map(f),
            old_generics: self.old_generics.map(f),
            new_generics: self.new_generics.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let stm = schema.get_type_mut(None, &self.type_name).ok_or_else(|| {
            ChangeSetError::InvalidAction {
                action: format!("edit generics"),
                reason: format!("no type named {} exists", self.type_name),
            }
        })?;

        let stm_type = stm.get_schema_type();

        let generics_opt = match stm {
            SchemaStm::Struct(s) => Some(&mut s.generics),
            SchemaStm::Enum(e) => Some(&mut e.generics),
            _ => None,
        };

        let genereics = generics_opt.ok_or_else(|| ChangeSetError::InvalidAction {
            action: format!("edit generics"),
            reason: format!(
                "attempted to change generics on {} which does not support generics",
                stm_type
            ),
        })?;

        if genereics != &self.old_generics {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit generics"),
                reason: format!(
                    "old generics of {} does not match, expected {} got {}",
                    self.type_name, self.old_generics, genereics
                ),
            });
        }

        *genereics = self.new_generics.clone();

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedGenerics<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (type_type, (old_generics, new_generics))) = context(
            "Parsing EditedGenerics",
            preceded(
                ws(char('*')),
                pair(
                    ws(Ident::ident),
                    key_value(Generics::parse, pair(char('='), char('>')), Generics::parse),
                ),
            ),
        )(s)?;

        Ok((
            s,
            EditedGenerics {
                type_name: type_type,
                old_generics,
                new_generics,
            },
        ))
    }
}

impl<I> ParserSerialize for EditedGenerics<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}* ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, " ")?;
        self.old_generics.compose(f, new_ctx)?;
        write!(f, " => ")?;
        self.new_generics.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for EditedGenerics<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {edited_generic_compose, EditedGenerics<I>}
