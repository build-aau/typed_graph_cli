use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::InputType;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;

use fake::Dummy;
use nom::error::context;
use nom::sequence::*;
use nom::character::complete::*;

use crate::{ChangeSetResult, ChangeSetError};

/// "+ (node|edge(\<end_points\>)|struct|enum) \<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedType<I> {
    pub comments: Comments,
    pub attributes: Attributes<I>,
    pub type_type: SchemaStmType,
    pub type_name: Ident<I>,
}

impl<I> EditedType<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EditedType<O> 
    where
        F: Fn(I) -> O + Copy
    {
        EditedType {
            comments: self.comments,
            attributes: self.attributes.map(f),
            type_name: self.type_name.map(f),
            type_type: self.type_type
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        let stm = schema.get_type_mut(Some(self.type_type), &self.type_name).ok_or_else(|| {
            ChangeSetError::InvalidAction { 
                action: format!("edit quantity"), 
                reason: format!("no edge type named {} exists", self.type_name) 
            }
        })?;

        let current_comments = match stm {
            SchemaStm::Node(n) => &mut n.comments,
            SchemaStm::Enum(n) => &mut n.comments,
            SchemaStm::Edge(n) => &mut n.comments,
            SchemaStm::Struct(n) => &mut n.comments,
            SchemaStm::Import(n) => &mut n.comments,
        };

        current_comments.replace_doc_comments(&self.comments);

        match stm {
            SchemaStm::Edge(e) => e.attributes = self.attributes.clone(),
            _ => ()
        }


        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedType<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, attributes, (type_type, type_name))) = context(
            "Parsing EditedType",
            tuple((
                Comments::parse,
                Attributes::parse,
                preceded(
                    ws(char('*')),
                    pair(
                        SchemaStmType::parse, 
                        ws(Ident::ident)
                    ),
                )
            ))
        )(s)?;

        Ok((
            s,
            EditedType {
                comments,
                attributes,
                type_type, 
                type_name
            },
        ))
    }
}

impl<I> ParserSerialize for EditedType<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.comments.compose(f)?;
        self.attributes.compose(f)?;
        write!(f, "* ")?;
        self.type_type.compose(f)?;
        write!(f, " ")?;
        self.type_name.compose(f)?;
        Ok(())
    }
}

impl<I> Display for EditedType<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string()
            .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test!{edited_type_compose, EditedType<I>}