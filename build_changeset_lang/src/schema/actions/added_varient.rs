use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

use crate::AddedVarientType;
use crate::{ChangeSetError, ChangeSetResult};

/// "+ enum \<ident\>.\<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct AddedVarient<I> {
    pub(crate) comments: Comments,
    pub(crate) type_name: Ident<I>,
    pub(crate) varient_name: Ident<I>,
    pub(crate) varient_type: AddedVarientType,
    pub(crate) order: u64
}

impl<I> AddedVarient<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedVarient<O>
    where
        F: Fn(I) -> O + Copy,
    {
        AddedVarient {
            comments: self.comments,
            type_name: self.type_name.map(f),
            varient_name: self.varient_name.map(f),
            varient_type: self.varient_type,
            order: self.order
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let enum_stm = schema
            .get_type_mut(Some(SchemaStmType::Enum), &self.type_name)
            .ok_or_else(|| ChangeSetError::InvalidAction {
                action: format!("add varient"),
                reason: format!("no enum type named {} exists", self.type_name),
            })?;

        if let SchemaStm::Enum(e) = enum_stm {
            let name_collision = e.get_varient(&self.varient_name);

            if name_collision.is_some() {
                return Err(ChangeSetError::InvalidAction {
                    action: format!("add varient"),
                    reason: format!("varient {} already exists", self.varient_name),
                });
            }

            let new_varient = match self.varient_type {
                AddedVarientType::Struct => EnumVarient::Struct { 
                    name: self.varient_name.clone(), 
                    comments: self.comments.get_doc_comments(), 
                    fields: Default::default(), 
                    marker: Mark::null()
                },
                AddedVarientType::Unit => EnumVarient::Unit { 
                    name: self.varient_name.clone(), 
                    comments: self.comments.get_doc_comments(), 
                    marker: Mark::null()
                },
            };
            e.varients.insert(self.order as usize, new_varient);
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedVarient<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, ((type_name, varient_name), order, varient_type))) = context(
            "Parsing AddedVarient",
            pair(
                Comments::parse,
                preceded(
                    ws(char('+')),
                    preceded(
                        ws(tag("enum")),
                        tuple((
                            separated_pair(Ident::ident, char('.'), Ident::ident),
                            surrounded('(', u64, ')'),
                            ws(AddedVarientType::parse)
                        ))
                    ),
                ),
            ),
        )(s)?;

        Ok((
            s,
            AddedVarient {
                comments,
                type_name,
                varient_name,
                varient_type,
                order
            },
        ))
    }
}

impl<I> ParserSerialize for AddedVarient<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        self.comments.compose(f, ctx)?;
        write!(f, "{indents}+ enum ")?;
        self.type_name.compose(f, new_ctx)?;
        write!(f, ".")?;
        self.varient_name.compose(f, new_ctx)?;
        write!(f, "({})", self.order)?;
        write!(f, " ")?;
        self.varient_type.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for AddedVarient<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {added_varient_compose, AddedVarient<I>}
