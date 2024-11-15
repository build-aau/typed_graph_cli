use std::fmt::Display;

use build_script_shared::compose_test;
use build_script_shared::error::ParserSlimResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::{Faker, Rng};

use crate::FieldPath;
use crate::{ChangeSetError, ChangeSetResult};
use build_script_lang::schema::*;
use fake::Dummy;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;

/// "* \<ident\>.\<ident\>: \<type\> => \<type\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct EditedOpaque<I> {
    pub(crate) field_path: FieldPath<I>,
    pub(crate) old_type: Types<I>,
    pub(crate) new_type: Types<I>,
}

impl<I> EditedOpaque<I> {
    pub fn old_type(&self) -> &Types<I> {
        &self.old_type
    }

    pub fn new_type(&self) -> &Types<I> {
        &self.new_type
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedOpaque<O>
    where
        F: Fn(I) -> O + Copy,
    {
        EditedOpaque {
            field_path: self.field_path.map(f),
            old_type: self.old_type.map(f),
            new_type: self.new_type.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq + Ord,
    {
        if self.field_path.path.is_empty() {
            return Err(ChangeSetError::InvalidAction {
                action: format!("retrieving opaque"),
                reason: format!("Attempted to resolve type path without type {}", self),
            });
        }

        let ty = schema
            .iter_mut()
            .find(|s| s.get_type() == &self.field_path.root)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("retrieving opaque"),
                reason: format!("Failed to find type for {}", self),
            })?;

        match ty {
            SchemaStm::Enum(e) => {
                if self.field_path.path.len() != 1 {
                    return Err(ChangeSetError::InvalidAction {
                        action: format!("retrieving opaque"),
                        reason: format!("Attempted to go deeper than possible {}", self),
                    });
                }

                let varient_name = &self.field_path.path[0];
                let varient = e.get_varient_mut(varient_name).ok_or_else(|| {
                    ChangeSetError::InvalidAction {
                        action: format!("retrieving opaque"),
                        reason: format!("Failed to find varient for {}", self),
                    }
                })?;

                match varient {
                    EnumVarient::Opaque { ty, .. } => {
                        if ty != &self.old_type {
                            return Err(ChangeSetError::InvalidAction {
                                action: format!("edit opaque"),
                                reason: format!(
                                    "old type of {} does not match, expected {} got {}",
                                    self.field_path, self.old_type, self.new_type
                                ),
                            });
                        }

                        *ty = self.new_type.clone();
                    }
                    _ => {
                        return Err(ChangeSetError::InvalidAction {
                            action: format!("retrieving opaque"),
                            reason: format!(
                                "Cannot retrieve opaque from other varients at {}",
                                self
                            ),
                        });
                    }
                };
            }
            SchemaStm::Edge(_)
            | SchemaStm::Node(_)
            | SchemaStm::Import(_)
            | SchemaStm::Struct(_) => {
                return Err(ChangeSetError::InvalidAction {
                    action: format!("retrieving opaque"),
                    reason: format!("Attempted to retrieve opaque {} from other type", self),
                })
            }
        };

        Ok(())
    }

    pub fn check_convertion_res(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.old_type.check_convertion_res(&self.new_type)?;
        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedOpaque<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (field_path, (old_type, new_type))) = context(
            "Parsing EditedOpaque",
            preceded(
                ws(char('*')),
                pair(
                    FieldPath::parse,
                    preceded(
                        ws(char(':')),
                        key_value(
                            Types::parse,
                            pair(char('='), char('>')),
                            Types::parse,
                        ),
                    ),
                ),
            ),
        )(s)?;

        Ok((
            s,
            EditedOpaque {
                field_path,
                old_type,
                new_type,
            },
        ))
    }
}

impl<I> ParserSerialize for EditedOpaque<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        write!(f, "{indents}* ")?;
        self.field_path.compose(f, new_ctx)?;
        write!(f, ": ")?;
        self.old_type.compose(f, new_ctx)?;
        write!(f, " => ")?;
        self.new_type.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for EditedOpaque<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for EditedOpaque<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let old_type = Types::dummy_with_rng(config, rng);
        let mut new_type = Types::dummy_with_rng(config, rng);

        // make sure a valid type is selected
        // This is a greedy approach to generating a type
        // but it stops us from having to maintain a generator for valid type pairs
        while !old_type.check_convertion(&new_type) {
            new_type = Types::dummy_with_rng(config, rng);
        }

        EditedOpaque {
            field_path: FieldPath::dummy_with_rng(config, rng),
            old_type,
            new_type,
        }
    }
}

compose_test! {edited_opaque_compose, EditedOpaque<I>}
