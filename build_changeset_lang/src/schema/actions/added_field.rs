use std::fmt::Display;

use build_script_lang::schema::FieldValue;
use build_script_lang::schema::Schema;
use build_script_lang::schema::Visibility;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

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
    pub(crate) order: u64
}

impl<I> AddedField<I> {
    pub fn field_type(&self) -> &Types<I> {
        &self.field_type
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> AddedField<O>
    where
        F: Fn(I) -> O + Copy,
    {
        AddedField {
            comments: self.comments,
            visibility: self.visibility,
            field_path: self.field_path.map(f),
            field_type: self.field_type.map(f),
            order: self.order
        }
    }

    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        let named_fields = self.field_path.retrieve_field(schema)?;

        let named_key = self.field_path.get_field_name_res()?;

        if named_fields.has_field(named_key.as_str()) {
            return Err(ChangeSetError::InvalidAction {
                action: format!("add field"),
                reason: format!("field at {} already exists", self.field_path),
            });
        }

        named_fields.insert_field(
            FieldValue {
                name: named_key.clone(),
                visibility: self.visibility,
                comments: self.comments.get_doc_comments(),
                field_type: self.field_type.clone(),
                order: self.order
            },
        );

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for AddedField<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, ((visibility, field_path), (field_type, order)))) = context(
            "Parsing AddedField",
            pair(
                Comments::parse,
                preceded(
                    ws(char('+')),
                    key_value(
                        pair(Visibility::parse, FieldPath::parse),
                        char(':'),
                        pair(Types::parse, surrounded('(', u64, ')')),
                    ),
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
                order
            },
        ))
    }
}

impl<I> ParserSerialize for AddedField<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        let new_ctx = ctx.set_indents(0);

        self.comments.compose(f, ctx)?;
        write!(f, "{indents}+ ")?;
        self.visibility.compose(f, new_ctx)?;
        self.field_path.compose(f, new_ctx)?;
        write!(f, ": ")?;
        self.field_type.compose(f, new_ctx)?;
        write!(f, "({})", self.order)?;
        Ok(())
    }
}

impl<I> Display for AddedField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {added_field_compose, AddedField<I>}
