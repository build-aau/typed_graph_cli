use super::{marked, Marked, Mark, ParserSerialize};
use crate::{InputType, compose_test};
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::{context, ContextError, ParseError};
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use std::fmt::Display;
use std::iter::once;
use fake::*;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Default)]
pub struct Ident<I> {
    name: String,
    marker: Mark<I>,
}

impl<I> Ident<I> {
    pub fn new<S>(name: S, marker: Mark<I>) -> Self
    where
        S: ToString,
    {
        Ident {
            name: name.to_string(),
            marker,
        }
    }

    pub fn new_alone<S>(name: S) -> Self 
    where
        I: Default,
        S: ToString
    {
        Ident {
            name: name.to_string(),
            marker: Mark::null()
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Ident<O>
    where
        F: FnMut(I) -> O,
    {
        Ident {
            name: self.name,
            marker: self.marker.map(f)
        }
    }

    /// Parse an identifyer starting with letters
    pub fn ident<E>(s: I) -> IResult<I, Ident<I>, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, (name, marker)) = marked(context(
            "Parsing Ident",
            recognize(pair(
                alt((
                    alpha1, 
                    tag("_"),
                    tag("-")
                )),
                many0_count(alt((
                    alphanumeric1, 
                    tag("_"),
                    tag("-")
                ))),
            )),
        ))(s)?;
    
        let ident = Ident {
            name: name.to_string(),
            marker,
        };
        Ok((s, ident))
    }
    
    /// Parse an identifyer that can start with a number
    pub fn ident_full<E>(s: I) -> IResult<I, Ident<I>, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, (name, marker)) = marked(context(
            "Parsing Full Ident",
            recognize(pair(
                    alt((
                        alphanumeric1,
                        tag("_"),
                        tag("-"),
                    )),
                many0_count(alt((
                    alphanumeric1, 
                    tag("_"),
                    tag("-"),
                    tag("."),
                ))),
            )),
        ))(s)?;
    
        let ident = Ident {
            name: name.to_string(),
            marker,
        };
        Ok((s, ident))
    }
}

impl<I> ParserSerialize for Ident<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

impl<I> Deref for Ident<I> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl<I> PartialEq for Ident<I> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<I> Eq for Ident<I> {}

impl<I> Display for Ident<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string())
    }
}

impl<I> Hash for Ident<I> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl<I> PartialOrd for Ident<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl<I> Ord for Ident<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl<I> Marked<I> for Ident<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

fn gen_alpha<R: Rng + ?Sized>(rng: &mut R) -> char {
    match rng.gen_range(0..=1) {
        0 => rng.gen_range('A'..='Z'),
        _ => rng.gen_range('a'..='z'),
    }
}

fn gen_alphanumeric<R: Rng + ?Sized>(rng: &mut R) -> char {
    match rng.gen_range(0..=2) {
        0 => rng.gen_range('A'..='Z'),
        1 => rng.gen_range('0'..='9'),
        _ => rng.gen_range('a'..='z'),
    }
}

fn gen_any_char<R: Rng + ?Sized>(rng: &mut R) -> char {
    match rng.gen_range(0..10) {
        0..=2 => rng.gen_range('A'..='Z'),
        3..=6 => rng.gen_range('a'..='z'),
        7 => *['-', '_'].choose(rng).unwrap() ,
        _ => rng.gen_range('0'..='9'),
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for Ident<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        Ident::dummy_with_rng(&SimpleIdentDummy, rng)
    }
}

const IDENT_DUMMY_LENGTH: usize = 10;

pub struct SimpleIdentDummy;
impl<I: Dummy<Faker>> Dummy<SimpleIdentDummy> for Ident<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &SimpleIdentDummy, rng: &mut R) -> Self {
        let len = rng.gen_range(6..IDENT_DUMMY_LENGTH);

        let s: String = once(gen_alpha(rng))
            .chain((0..len).map(|_| gen_any_char(rng))).collect();

        Ident { 
            name: s.clone(), 
            marker: Mark::new(I::dummy_with_rng(&Faker, rng))
        }
    }
}

pub struct FullIdentDummy;
impl<I: Dummy<Faker>> Dummy<FullIdentDummy> for Ident<I> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &FullIdentDummy, rng: &mut R) -> Self {
        let len = rng.gen_range(6..IDENT_DUMMY_LENGTH);

        let s: String = once(gen_alphanumeric(rng))
            .chain((0..len).map(|_| gen_any_char(rng))).collect();

        Ident { 
            name: s.clone(), 
            marker: Mark::dummy_with_rng(&Faker, rng)
        }
    }
}

compose_test!{ident_compose, Ident<I> with parser Ident::ident}
compose_test!{ident_full_compose, Ident<I> with parser Ident::ident_full}