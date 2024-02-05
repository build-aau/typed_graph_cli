use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::fmt::Display;

use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use nom::multi::many0;

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
pub struct EditedVariantsOrder<I> {
    pub(crate) type_name: Ident<I>,
    pub(crate) old_order: Vec<Ident<I>>,
    pub(crate) new_order: Vec<Ident<I>>,
}

impl<I> EditedVariantsOrder<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedVariantsOrder<O>
    where
        F: Fn(I) -> O + Copy,
    {
        EditedVariantsOrder {
            type_name: self.type_name.map(f),
            old_order: self.old_order
                .into_iter()
                .map(|v| v.map(f))
                .collect(),
            new_order: self.new_order
                .into_iter()
                .map(|v| v.map(f))
                .collect(),

        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let ty = schema.get_type_mut(Some(SchemaStmType::Enum), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("edit varients order"),
                reason: format!("Failed to find enum type {}", self.type_name),
            })?;
        
        if let SchemaStm::Enum(e) = ty {
            let old_order: Vec<_> = e.varients
                .iter()
                .map(|v| v.name())
                .cloned()
                .collect();

            /*
            if old_order != self.old_order {
                let fmt_actual_old_order = old_order
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                let fmt_expected_old_order = self.old_order
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(ChangeSetError::InvalidAction {
                    action: format!("edit varients order"),
                    reason: format!(
                        "old order of {} does not match, expected [{}] got [{}]",
                        e.name, 
                        fmt_expected_old_order,
                        fmt_actual_old_order

                    ),
                });
            }
             */

            // Create a map for looking up the position of the varient
            let new_order_map: HashMap<_, _> = self.new_order
                .iter()
                .enumerate()
                .map(|(i, v)| (v, i))
                .collect();

            // We might encounter to be deleted vairents
            // So to handle this we group all unknown at 0
            let default_order = 0;
            e.varients.sort_by_key(|v| new_order_map.get(v.name()).unwrap_or_else(|| &default_order));

        } else {
            return Err(ChangeSetError::InvalidAction {
                action: format!("edit varients order"),
                reason: format!(
                    "{} is not an enum",
                    self.type_name, 
                ),
            });
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedVariantsOrder<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (type_type, (old_order, new_order))) =
            context(
                "Parsing EditedVariantsOrder",
                preceded(
                    ws(char('*')),
                    tuple((
                        Ident::ident,
                        key_value(
                            surrounded('[', punctuated(Ident::ident, ','), ']'),
                            pair(char('='), char('>')),
                            surrounded('[', punctuated(Ident::ident, ','), ']')
                        )
                    )),
                ),
            )(s)?;

        Ok((
            s,
            EditedVariantsOrder {
                type_name: type_type,
                old_order: old_order,
                new_order: new_order
            },
        ))
    }
}

impl<I> ParserSerialize for EditedVariantsOrder<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}* ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, " ")?;
        write!(f, "[")?;
        let mut first = true;
        for name in &self.old_order {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            write!(f, "{name}")?;
        }
        write!(f, "]")?;
        write!(f, " => ")?;
        write!(f, "[")?;
        let mut first = true;
        for name in &self.new_order {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            write!(f, "{name}")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<I> Display for EditedVariantsOrder<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {edited_varient_order_compose, EditedVariantsOrder<I>}
