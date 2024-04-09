use build_script_shared::compose_test;
use build_script_shared::error::ParserResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Faker;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::*;
use nom::multi::*;
use nom::sequence::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(
    PartialEq, Eq, Debug, Hash, Clone, Default, PartialOrd, Ord, Dummy, Serialize, Deserialize,
)]
#[serde(bound = "I: Default + Clone")]
pub struct ImportExp<I> {
    pub name: Ident<I>,
    #[serde(flatten)]
    pub comments: Comments,
    #[serde(skip)]
    marker: Mark<I>,
}

impl<I> ImportExp<I> {
    pub fn new(name: Ident<I>, comments: Comments, marker: Mark<I>) -> Self {
        ImportExp {
            name,
            comments,
            marker,
        }
    }

    pub fn strip_comments(&mut self) {
        self.comments.strip_comments();
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> ImportExp<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        ImportExp {
            name: self.name.map(f),
            comments: self.comments,
            marker: self.marker.map(f),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for ImportExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("import"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context("Parsing import name", ws(cut(marked(Ident::ident))))(s)?;

        Ok((
            s,
            ImportExp {
                name,
                comments,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for ImportExp<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        let indents = ctx.create_indents();
        self.comments.compose(f, ctx)?;
        write!(f, "{indents}import ")?;
        self.name.compose(f, ctx.set_indents(0))?;

        Ok(())
    }
}

impl<I> Marked<I> for ImportExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

pub(crate) struct ImportExpOfType<I> {
    pub name: Ident<I>,
}

impl<I: Dummy<Faker> + Clone> Dummy<ImportExpOfType<I>> for ImportExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(
        config: &ImportExpOfType<I>,
        rng: &mut R,
    ) -> Self {
        let mut exp = ImportExp::dummy_with_rng(&Faker, rng);

        // Se the name to the expected value
        exp.name = config.name.clone();

        exp
    }
}

compose_test! {import_compose, ImportExp<I>}
