use super::Quantifier;
use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Faker;
use nom::character::complete::*;
use nom::sequence::pair;
use nom::Err;
use rand::Rng;
use std::collections::BTreeMap;
use std::collections::HashSet;

const ALLOWED_ATTRIBUTES: &[&str] = &["rename_inc", "rename_out"];

#[derive(Debug, Hash, PartialEq, Eq, Clone, Dummy, PartialOrd, Ord)]
pub struct EndPoint<I> {
    pub source: Ident<I>,
    pub target: Ident<I>,
    pub quantity: Quantifier<I>,
    #[dummy(faker = "AllowedKeyValueAttribute(ALLOWED_ATTRIBUTES)")]
    pub attributes: Attributes<I>,
    marker: Mark<I>,
}

impl<I> EndPoint<I> {
    pub fn new(
        quantity: Quantifier<I>,
        attributes: Attributes<I>,
        source: Ident<I>,
        target: Ident<I>,
        marker: Mark<I>,
    ) -> EndPoint<I> {
        EndPoint {
            quantity,
            attributes,
            source,
            target,
            marker,
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EndPoint<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        EndPoint {
            quantity: self.quantity.map(f),
            source: self.source.map(f),
            target: self.target.map(f),
            attributes: self.attributes.map(f),
            marker: self.marker.map(f),
        }
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.attributes.check_key_value(ALLOWED_ATTRIBUTES)?;

        Ok(())
    }

    pub fn check_types(
        &self,
        _all_reference_types: &HashSet<Ident<I>>,
        node_reference_types: &HashSet<Ident<I>>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        if !node_reference_types.contains(&self.source) {
            return Err(Err::Failure(ParserError::new_at(
                self,
                ParserErrorKind::UnknownReference(self.source.to_string()),
            )));
        }
        if !node_reference_types.contains(&self.target) {
            return Err(Err::Failure(ParserError::new_at(
                self,
                ParserErrorKind::UnknownReference(self.target.to_string()),
            )));
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EndPoint<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (attributes, ((source, (quantity, target)), marker))) = pair(
            Attributes::parse,
            marked(key_value(
                Ident::ident,
                ws(pair(char('='), char('>'))),
                pair(Quantifier::parse, Ident::ident),
            )),
        )(s)?;

        Ok((
            s,
            EndPoint {
                quantity,
                attributes,
                source,
                target,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for EndPoint<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        self.attributes.compose(f)?;
        self.source.compose(f)?;
        write!(f, " =>")?;
        self.quantity.compose(f)?;
        write!(f, " ")?;
        self.target.compose(f)?;
        Ok(())
    }
}

impl<I: Default> Default for EndPoint<I> {
    fn default() -> Self {
        EndPoint::new(
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Mark::null(),
        )
    }
}

impl<I> Marked<I> for EndPoint<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

pub struct EndpointMap;
impl<I: Dummy<Faker>> Dummy<EndpointMap> for BTreeMap<(Ident<I>, Ident<I>), EndPoint<I>> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &EndpointMap, rng: &mut R) -> Self {
        let endpoints  = Vec::<EndPoint<I>>::dummy_with_rng(&Faker, rng);
        endpoints
            .into_iter()
            .map(|endpoint| (
                (
                    Ident::new(&endpoint.source, Mark::dummy_with_rng(&Faker, rng)),
                    Ident::new(&endpoint.target, Mark::dummy_with_rng(&Faker, rng))
                ),
                endpoint
            ))
            .collect()
    }
}

compose_test! {endpoint_compose, EndPoint<I>}
