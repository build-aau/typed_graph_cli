use std::fmt::Display;
use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::parsers::*;
use build_script_shared::InputType;

use fake::Dummy;
use nom::bytes::complete::tag;
use nom::character::complete::*;
use nom::error::context;
use nom::sequence::*;

use crate::ChangeSetResult;

/// "\<attributes\>
/// * (node|edge(\<end_points\>)|struct|enum) \<ident\>"
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub struct EditedSchema {
    pub comments: Comments,
}

impl EditedSchema {
    pub fn apply<I>(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq,
    {
        schema.comments = self.comments.clone();

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EditedSchema {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, (comments, _)) = context(
            "Parsing EditedSchema",
            pair(
                Comments::parse,
                preceded(ws(char('*')), tag("schema")),
            ),
        )(s)?;

        Ok((
            s,
            EditedSchema {
                comments,
            },
        ))
    }
}

impl ParserSerialize for EditedSchema {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();

        self.comments.compose(f, ctx)?;
        write!(f, "{indents}* schema")?;
        Ok(())
    }
}

impl Display for EditedSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ser = self.serialize_to_string().map_err(|_| std::fmt::Error)?;
        write!(f, "{}", ser)
    }
}

compose_test! {edited_schema_compose, EditedSchema}
