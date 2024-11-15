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
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashSet;

const RENAME_INC: &'static str = "rename_inc";
const RENAME_OUT: &'static str = "rename_out";

const ALLOWED_KEY_ATTRIBUTES: &[&str] = &[RENAME_INC, RENAME_OUT];

#[derive(Debug, Hash, PartialEq, Eq, Clone, Dummy, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub struct EndPoint<I> {
    pub source: Ident<I>,
    pub target: Ident<I>,
    pub incoming_quantity: Quantifier<I>,
    pub outgoing_quantity: Quantifier<I>,
    #[dummy(faker = "AllowedKeyValueAttribute(ALLOWED_KEY_ATTRIBUTES)")]
    #[serde(flatten)]
    pub attributes: Attributes<I>,
    #[serde(skip)]
    marker: Mark<I>,
}

impl<I> EndPoint<I> {
    pub fn new(
        incoming_quantity: Quantifier<I>,
        outgoing_quantity: Quantifier<I>,
        attributes: Attributes<I>,
        source: Ident<I>,
        target: Ident<I>,
        marker: Mark<I>,
    ) -> EndPoint<I> {
        EndPoint {
            incoming_quantity,
            outgoing_quantity,
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
            incoming_quantity: self.incoming_quantity.map(f),
            outgoing_quantity: self.outgoing_quantity.map(f),
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
        self.attributes
            .check_attributes(ALLOWED_KEY_ATTRIBUTES, &[], &[])?;

        Ok(())
    }

    pub fn get_rename_inc(&self) -> Option<&str> {
        self.attributes
            .get_key_value(RENAME_INC)
            .map(|kv| kv.value.as_ref())
    }

    pub fn get_rename_out(&self) -> Option<&str> {
        self.attributes
            .get_key_value(RENAME_OUT)
            .map(|kv| kv.value.as_ref())
    }

    pub fn check_types(&self, node_reference_types: &HashSet<Ident<I>>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        if !node_reference_types.contains(&self.source) {
            return Err(Err::Failure(ParserError::new_at(
                self.source.marker(),
                ParserErrorKind::UnknownReference(self.source.to_string()),
            )));
        }
        if !node_reference_types.contains(&self.target) {
            return Err(Err::Failure(ParserError::new_at(
                self.target.marker(),
                ParserErrorKind::UnknownReference(self.target.to_string()),
            )));
        }

        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for EndPoint<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (attributes, (((source, outgoing_quantity), (target, incoming_quantity)), marker))) =
            pair(
                Attributes::parse,
                marked(key_value(
                    pair(Ident::ident, Quantifier::parse),
                    ws(pair(char('='), char('>'))),
                    pair(Ident::ident, Quantifier::parse),
                )),
            )(s)?;

        Ok((
            s,
            EndPoint {
                attributes,
                source,
                outgoing_quantity,
                incoming_quantity,
                target,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for EndPoint<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> ComposerResult<()> {
        let endpoint_ctx = ctx.set_indents(0);

        self.attributes.compose(f, ctx)?;
        self.source.compose(f, ctx)?;
        self.outgoing_quantity.compose(f, endpoint_ctx)?;
        write!(f, " => ")?;
        self.target.compose(f, endpoint_ctx)?;
        self.incoming_quantity.compose(f, endpoint_ctx)?;
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
        let endpoints = Vec::<EndPoint<I>>::dummy_with_rng(&Faker, rng);
        endpoints
            .into_iter()
            .map(|endpoint| {
                (
                    (
                        Ident::new(&endpoint.source, Mark::dummy_with_rng(&Faker, rng)),
                        Ident::new(&endpoint.target, Mark::dummy_with_rng(&Faker, rng)),
                    ),
                    endpoint,
                )
            })
            .collect()
    }
}

compose_test! {endpoint_compose, EndPoint<I>}
