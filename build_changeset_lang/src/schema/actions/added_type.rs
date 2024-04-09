use std::fmt::Display;

use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

use crate::{AddedTypeData, ChangeSetError, ChangeSetResult};

/// "+ (node|edge(\<end_points\>)|struct|enum) \<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct AddedType<I> {
    pub comments: Comments,
    pub attributes: Attributes<I>,
    pub type_type: AddedTypeData<I>,
    pub type_name: Ident<I>,
}

impl<I> AddedType<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedType<O>
    where
        F: Fn(I) -> O + Copy,
    {
        AddedType {
            comments: self.comments,
            attributes: self.attributes.map(f),
            type_name: self.type_name.map(f),
            type_type: self.type_type.map(f),
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let ty = self.type_type.get_type();
        let name_collision = schema.get_type(Some(ty), &self.type_name).is_some();
        if name_collision {
            return Err(ChangeSetError::InvalidAction {
                action: format!("add {} {}", ty, self.type_name),
                reason: format!("{} with same name already exists", ty),
            });
        }

        let new_content = match &self.type_type {
            AddedTypeData::Node => SchemaStm::Node(NodeExp::new(
                self.comments.get_doc_comments(),
                self.attributes.clone(),
                self.type_name.clone(),
                Fields::default(),
                Mark::default(),
            )),
            AddedTypeData::Struct => SchemaStm::Struct(StructExp::new(
                self.comments.get_doc_comments(),
                self.attributes.clone(),
                self.type_name.clone(),
                Default::default(),
                Fields::default(),
                Mark::default(),
            )),
            AddedTypeData::Edge { endpoints } => SchemaStm::Edge(EdgeExp::new(
                self.comments.get_doc_comments(),
                self.attributes.clone(),
                self.type_name.clone(),
                Fields::default(),
                endpoints.clone(),
                Mark::default(),
            )),
            AddedTypeData::Enum => SchemaStm::Enum(EnumExp::new(
                self.comments.get_doc_comments(),
                self.attributes.clone(),
                self.type_name.clone(),
                Default::default(),
                Default::default(),
                Mark::null(),
            )),
            AddedTypeData::Import => SchemaStm::Import(ImportExp::new(
                self.type_name.clone(),
                Default::default(),
                Mark::null(),
            )),
        };

        schema.push(new_content);

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedType<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, attributes, (type_type, type_name))) = context(
            "Parsing AddedType",
            tuple((
                Comments::parse,
                Attributes::parse,
                preceded(ws(char('+')), pair(AddedTypeData::parse, ws(Ident::ident))),
            )),
        )(s)?;

        Ok((
            s,
            AddedType {
                comments,
                attributes,
                type_type,
                type_name,
            },
        ))
    }
}

impl<I> ParserSerialize for AddedType<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        self.comments.compose(f, ctx)?;
        self.attributes.compose(f, ctx)?;
        write!(f, "{indents}+ ")?;
        self.type_type.compose(f, new_ctx)?;
        write!(f, " ")?;
        self.type_name.compose(f, new_ctx)?;
        Ok(())
    }
}

impl<I> Display for AddedType<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {added_type_compose, AddedType<I>}
