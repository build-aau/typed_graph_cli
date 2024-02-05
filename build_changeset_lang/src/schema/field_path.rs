use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::error::ParserResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use nom::character::complete::char;
use nom::combinator::cut;
use nom::error::context;
use nom::sequence::{pair, preceded};
use std::fmt::Display;

use crate::{ChangeSetResult, ChangeSetError};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Dummy)]
pub struct FieldPath<I> {
    pub root: Ident<I>,
    pub path: Vec<Ident<I>>,
}

impl<I> FieldPath<I> {
    pub fn new(root: Ident<I>) -> FieldPath<I> {
        FieldPath {
            root,
            path: Vec::new(),
        }
    }

    pub fn new_path(root: Ident<I>, path: Vec<Ident<I>>) -> FieldPath<I> {
        FieldPath { root, path }
    }

    pub fn get_field_name(&self) -> Option<&Ident<I>> {
        self.path.last()
    }

    pub fn get_field_name_res(&self) -> ChangeSetResult<&Ident<I>> {
        self.get_field_name()
            .ok_or_else(|| {
                ChangeSetError::InvalidAction {
                    action: format!("retrieving field name"),
                    reason: format!("Failed to find field name in path {}", self),
                }
            })
    }

    pub fn retrieve_field<'a>(
        &'a self,
        schema: &'a mut Schema<I>,
    ) -> ChangeSetResult<&'a mut Fields<I>> {
        if self.path.is_empty() {
            return Err(ChangeSetError::InvalidAction {
                action: format!("retrieving fields"),
                reason: format!("Attempted to resolve type path without type {}", self),
            });
        }

        let ty = schema
            .content
            .iter_mut()
            .find(|s| s.get_type() == &self.root)
            .ok_or_else(|| {
                ChangeSetError::InvalidAction {
                    action: format!("retrieving fields"),
                    reason: format!("Failed to find type for {}", self),
                }
            })?;
            
        if self.path.len() == 1 {
            let fields = ty.get_fields_mut()
                .ok_or_else(|| {
                    ChangeSetError::InvalidAction {
                        action: format!("retrieving fields"),
                        reason: format!("Failed to find fields in type for {}", self),
                    }
                })?;

            return Ok(fields);
        }

        match ty {
            SchemaStm::Enum(e) => {
                if self.path.len() != 2 {
                    return Err(ChangeSetError::InvalidAction {
                        action: format!("retrieving fields"),
                        reason: format!("Failed to resolve {} to field in enum since it is to long", self),
                    })
                }
                let varient_name = &self.path[0];
                let varient = e.get_varient_mut(varient_name)
                    .ok_or_else(|| {
                        ChangeSetError::InvalidAction {
                            action: format!("retrieving fields"),
                            reason: format!("Failed to find varient for {}", self),
                        }
                    })?;
                
                let varient_fields = match varient {
                    EnumVarient::Struct { fields, .. } => Ok(fields),
                    EnumVarient::Unit { .. } => Err(ChangeSetError::InvalidAction {
                        action: format!("retrieving fields"),
                        reason: format!("Cannot retrieve fields from unit varient at {}", self),
                    }),
                };

                return varient_fields;
            }
            SchemaStm::Edge(_)
            | SchemaStm::Node(_)
            | SchemaStm::Import(_)
            | SchemaStm::Struct(_) => {
                return Err(ChangeSetError::InvalidAction {
                    action: format!("retrieving fields"),
                    reason: format!("Attempted to retrieve field {} from a to shallow type", self),
                })
            }
        };

        /*
        Err(ChangeSetError::InvalidAction {
            action: format!("retrieving fields"),
            reason: format!("Failed to find filed at {}", self),
        })
        */
        
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> FieldPath<O>
    where
        F: Fn(I) -> O + Copy,
    {
        FieldPath {
            root: self.root.map(f),
            path: self.path.into_iter().map(|p| p.map(f)).collect(),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for FieldPath<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (root, path)) = context(
            "Parsing FieldPath",
            pair(
                Ident::ident,
                preceded(ws(char('.')), cut(punctuated(Ident::ident, '.'))),
            ),
        )(s)?;

        Ok((s, FieldPath { root, path }))
    }
}

impl<I> ParserSerialize for FieldPath<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let field_path_ctx = ctx.set_indents(0);
        self.root.compose(f, ctx)?;
        write!(f, ".")?;
        let iter = self.path.iter().enumerate();
        for (i, seg) in iter {
            seg.compose(f, field_path_ctx)?;
            if i + 1 != self.path.len() {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

impl<I> Display for FieldPath<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)?;
        for seg in &self.path {
            write!(f, ".{}", seg)?;
        }

        Ok(())
    }
}

impl<I> FieldPath<I>
where
    I: Clone,
{
    pub(crate) fn push(&self, head: Ident<I>) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(head);
        FieldPath {
            root: self.root.clone(),
            path: new_path,
        }
    }
}

compose_test! {field_path_compose, FieldPath<I>}
