use std::fmt::{Display, Write};

use crate::error::{ComposerResult, ParserResult};
use crate::parsers::{marked, punctuated, surrounded, ComposeContext, Ident, Mark, Marked, ParserDeserialize, ParserSerialize};
use crate::{compose_test, InputType};
use fake::{Dummy, Faker};
use nom::combinator::opt;
use nom::error::context;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

#[derive(Eq, Debug, Hash, Default, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct Generics<I> {
    pub generics: Vec<Generic<I>>
}

#[derive(Eq, Debug, Hash, Default, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct Generic<I> {
    pub letter: Ident<I>,
    #[serde(skip)]
    pub marker: Mark<I>
}

impl<I> Generics<I> {
    pub fn map<O, F>(self, f: F) -> Generics<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Generics { 
            generics: self.generics
                .into_iter()
                .map(|g| g.map(f))
                .collect()
        }
    }

    pub fn get_meta(&self) -> Vec<String> {
        self
            .generics
            .iter()
            .map(Generic::get_meta)
            .collect()
    }
}

impl<I> Generic<I> {
    pub fn map<O, F>(self, f: F) -> Generic<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        Generic {
            letter: self.letter.map(f),
            marker: self.marker.map(f)
        }
    }

    pub fn get_meta(&self) -> String {
        self.letter.to_string()
    }
}

impl<I> Marked<I> for Generic<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

impl<I> PartialEq for Generics<I> {
    fn eq(&self, other: &Self) -> bool {
        self.generics.eq(&other.generics)
    }
}

impl<I> PartialEq for Generic<I> {
    fn eq(&self, other: &Self) -> bool {
        self.letter.eq(&other.letter)
    }
}

impl<I: InputType> ParserDeserialize<I> for Generics<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, generics) = context(
            "Parsing Generics", 
            opt(surrounded('<', punctuated(Generic::parse, ','), '>'))
        )(s)?;

        Ok((
            s, 
            Generics {
                generics: generics.unwrap_or_default()
            }
        ))
    }
}

impl<I: InputType> ParserDeserialize<I> for Generic<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (letter, marker)) = context("Parsing Generic", marked(Ident::ident))(s)?;

        Ok((
            s,
            Generic {
                letter,
                marker
            }
        ))
    }
}

impl<I> ParserSerialize for Generics<I> {
    fn compose<W: Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        if self.generics.is_empty() {
            return Ok(());
        }

        write!(f, "<")?;
        let mut first = true;
        for generic in &self.generics {
            if !first {
                write!(f, ", ")?;
            } else {
                first = false;
            }
            generic.compose(f, ctx)?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

impl<I> ParserSerialize for Generic<I> {
    fn compose<W: Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        self.letter.compose(f, ctx)
    }
}

impl<I> Display for Generics<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize_to_string().unwrap())
    }
}

pub struct GenericsOfSize(pub usize);

impl<I: Dummy<Faker>> Dummy<GenericsOfSize> for Generics<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &GenericsOfSize, rng: &mut R) -> Self {
        Generics { 
            generics: (0..config.0)
                .map(|i| Generic::dummy_with_rng(&GenericAtNumber(i), rng))
                .collect() 
        }
    }
}


impl<I: Dummy<Faker>> Dummy<Faker> for Generics<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        Generics::dummy_with_rng(&GenericsOfSize(rng.gen_range(0..5)), rng)
    }
}

pub struct GenericAtNumber(pub usize);

impl<I: Dummy<Faker>> Dummy<GenericAtNumber> for Generic<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &GenericAtNumber, rng: &mut R) -> Self {

        Generic { 
            letter: Ident::new(
                ('A'..'Z')
                    .skip(config.0)
                    .next()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| format!("T{}", config.0 - ('A'..'Z').count())), 
                Mark::dummy_with_rng(&Faker, rng)
            ), 
            marker: Mark::dummy_with_rng(&Faker, rng)
        }
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for Generic<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        Generic::dummy_with_rng(&GenericAtNumber(rng.gen_range(0..5)), rng)
    }
}


compose_test! {generics_compose, Generics<I>}

compose_test! {generic_compose, Generic<I>}