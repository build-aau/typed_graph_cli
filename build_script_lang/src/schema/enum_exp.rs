use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Faker;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::*;
use nom::multi::*;
use nom::sequence::*;
use nom::Err;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::hash::Hash;
use fake::Dummy;

#[derive(PartialEq, Eq, Debug, Clone, Default, PartialOrd, Ord)]
pub struct EnumExp<I> {
    pub name: Ident<I>,
    pub comments: Comments,
    pub varients: BTreeMap<Ident<I>, Comments>,
    marker: Mark<I>,
}

impl<I> EnumExp<I> {
    pub fn new(comments: Comments, name: Ident<I>, varients: BTreeMap<Ident<I>, Comments>, marker: Mark<I>) -> Self {
        EnumExp {
            comments,
            name,
            varients,
            marker,
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EnumExp<O> 
    where
        F: FnMut(I) -> O + Copy,
    {
        EnumExp {
            comments: self.comments,
            name: self.name.map(f),
            varients: self.varients.into_iter().map(|(var, comments)| (var.map(f), comments)).collect(),
            marker: self.marker.map(f)
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for EnumExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("enum"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context(
            "Parsing Enum type", 
            ws(cut(marked(Ident::ident)))
        )(s)?;
        // Parse the list of fields
        let (s, varients) = owned_context(
            format!("Parsing {}", name),
            cut(surrounded('{', punctuated(pair(Comments::parse, Ident::ident), ','), '}')),
        )(s)?;

        let mut varients_checker: BTreeSet<&Ident<I>> = BTreeSet::new();
        for (_, varient) in &varients {
            let first = varients_checker.iter().find(|v| varient.cmp(v).is_eq()).cloned();
            if !varients_checker.insert(varient) {
                let first = first.unwrap();
                return Err(Err::Failure(
                    vec![
                        (varient.marker(), ParserErrorKind::DuplicateDefinition(varient.to_string())),
                        (first.marker(), ParserErrorKind::FirstOccurance),
                    ].into_iter().collect()
                ));
            }
        }

        Ok((
            s,
            EnumExp {
                comments,
                name,
                varients: varients.into_iter().map(|(a, b)| (b, a)).collect(),
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for EnumExp<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        self.comments.compose(f)?;
        write!(f, "enum ")?;
        self.name.compose(f)?;
        writeln!(f, " {{")?;
        let varient_iter = self.varients.iter().enumerate();
        let mut first = true;
        for (_, (varient, comments)) in varient_iter {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            comments.compose(f)?;
            varient.compose(f)?;
            writeln!(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<I> Marked<I> for EnumExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I> Hash for EnumExp<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        let varient: BTreeSet<_> = self.varients.iter().collect();
        varient.hash(state);
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for EnumExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        let len = rng.gen_range(0..10);

        let mut taken_names = HashSet::new();
        let mut fields = Vec::new();

        let mut i = 0;
        while i < len {
            let new_name: Ident<I> = Ident::dummy_with_rng(&Faker, rng);
            if taken_names.contains(&*new_name) {
                continue;
            }

            taken_names.insert(new_name.to_string());
            fields.push(new_name);
            i += 1;

        }

        EnumExp {
            name: Ident::dummy_with_rng(&Faker, rng),
            comments: Comments::dummy_with_rng(&Faker, rng),

            varients: fields.into_iter().map(|name| (Ident::new(name, Mark::dummy_with_rng(&Faker, rng)), Comments::dummy_with_rng(&Faker, rng))).collect(), 
            marker: Mark::dummy_with_rng(&Faker, rng)
        }
    }
}

pub(crate) struct EnumExpOfType<I> {
    pub name: Ident<I>
}

impl<I: Dummy<Faker> + Clone> Dummy<EnumExpOfType<I>> for EnumExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &EnumExpOfType<I>, rng: &mut R) -> Self {
        let mut exp = EnumExp::dummy_with_rng(&Faker, rng);
        
        // Se the name to the expected value
        exp.name = config.name.clone();

        exp
    }
}

compose_test!{enum_compose, EnumExp<I>}