use std::collections::HashSet;

use super::FieldWithReferences;
use super::Fields;
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

#[derive(PartialEq, Eq, Debug, Hash, Clone, Default, PartialOrd, Ord, Dummy)]
pub struct StructExp<I> {
    pub name: Ident<I>,
    pub comments: Comments,
    pub fields: Fields<I>,
    marker: Mark<I>,
}

impl<I> StructExp<I> {
    pub fn new(comments: Comments, name: Ident<I>, fields: Fields<I>, marker: Mark<I>) -> Self {
        StructExp {
            comments,
            name,
            fields,
            marker,
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> StructExp<O> 
    where
        F: FnMut(I) -> O + Copy,
    {
        StructExp {
            comments: self.comments,
            name: self.name.map(f),
            fields: self.fields.map(f),
            marker: self.marker.map(f)
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for StructExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("struct"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context(
            "Parsing Struct type", 
            ws(cut(marked(Ident::ident)))
        )(s)?;
        // Parse the list of fields
        let (s, fields) = owned_context(
            format!("Parsing {}", name),
            cut(opt(Fields::parse)),
        )(s)?;

        Ok((
            s,
            StructExp {
                comments,
                name,
                fields: fields.unwrap_or_default(),
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for StructExp<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.comments.compose(f)?;
        write!(f, "struct ")?;
        self.name.compose(f)?;
        write!(f, " ")?;
        self.fields.compose(f)?;

        Ok(())
    }
}

impl<I> Marked<I> for StructExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

pub(crate) struct StructExpOfType<I> {
    pub name: Ident<I>,
    pub ref_types: HashSet<String>
}

impl<I: Dummy<Faker> + Clone> Dummy<StructExpOfType<I>> for StructExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &StructExpOfType<I>, rng: &mut R) -> Self {
        let mut exp = StructExp::dummy_with_rng(&Faker, rng);
        
        // Se the name to the expected value
        exp.name = config.name.clone();

        // Make sure all type references point to existing types
        exp.fields = Fields::dummy_with_rng(&FieldWithReferences(config.ref_types.clone()), rng);

        exp
    }
}

compose_test!{struct_compose, StructExp<I>}