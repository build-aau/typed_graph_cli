use std::collections::BTreeMap;
use std::collections::HashSet;
use super::EndPoint;
use super::FieldWithReferences;
use super::Fields;
use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Faker;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::*;
use fake::Dummy;
use rand::seq::IteratorRandom;
use crate::schema::EndpointMap;

const ALLOWED_ATTRIBUTES: &[&str] = &["rename_inc", "rename_out"];

#[derive(PartialEq, Eq, Debug, Hash, Clone, Default, PartialOrd, Ord, Dummy)]
pub struct EdgeExp<I> {
    pub name: Ident<I>,
    pub comments: Comments,
    #[dummy(faker = "AllowedKeyValueAttribute(ALLOWED_ATTRIBUTES)")]
    pub attributes: Attributes<I>,
    pub fields: Fields<I>,
    #[dummy(faker = "EndpointMap")]
    pub endpoints: BTreeMap<(Ident<I>, Ident<I>), EndPoint<I>>,
    marker: Mark<I>,
}

impl<I> EdgeExp<I> {
    pub fn new(
        comments: Comments,
        attributes: Attributes<I>,
        name: Ident<I>,
        fields: Fields<I>,
        endpoints: BTreeMap<(Ident<I>, Ident<I>), EndPoint<I>>,
        marker: Mark<I>,
    ) -> Self {
        EdgeExp {
            comments,
            attributes,
            name,
            fields,
            endpoints,
            marker,
        }
    }

    pub fn parse_endpoints(s: I) -> ParserResult<I, BTreeMap<(Ident<I>, Ident<I>), EndPoint<I>>>
    where
        I: InputType,
    {
        let (s, endpoints) = ws(surrounded(
            '(',
            punctuated(EndPoint::parse, ','),
            ')',
        ))(s)?;

        let mut final_endpoints = BTreeMap::new();
        for endpoint in endpoints {
            let key = (endpoint.source.clone(), endpoint.target.clone());
            if final_endpoints.contains_key(&key) {
                return Err(Err::Failure(ParserError::new_at(&endpoint, ParserErrorKind::DuplicateDefinition(format!("{} -> {}", endpoint.source, endpoint.target)))))
            }
            final_endpoints.insert(key, endpoint);
        }

        Ok((s, final_endpoints))
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.attributes.check_key_value(ALLOWED_ATTRIBUTES)?;

        for endpoint in self.endpoints.values() {
            endpoint.check_attributes()?;
        }

        Ok(())
    }

    pub fn check_types(&self, all_reference_types: &HashSet<Ident<I>>, node_reference_types: &HashSet<Ident<I>>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.fields.check_types(all_reference_types)?;
        for endpoint in self.endpoints.values() {
            endpoint.check_types(all_reference_types, node_reference_types)?;
        }
        Ok(())
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EdgeExp<O> 
    where
        F: FnMut(I) -> O + Copy,
    {
        EdgeExp {
            comments: self.comments,
            attributes: self.attributes.map(f),
            name: self.name.map(f),
            fields: self.fields.map(f),
            endpoints: self.endpoints
                .into_iter()
                .map(|((source, target), endpoint)| ((source.map(f), target.map(f)), endpoint.map(f)))
                .collect(),
            marker: self.marker.map(f)
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for EdgeExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, attributes) = Attributes::parse(s)?;
        // Parse edge keyword
        let (s, _) = ws(terminated(tag("edge"), multispace1))(s)?;
        // Parse the name
        let (s, (name, marker)) = context(
            "Parsing Edge type", 
            ws(cut(marked(Ident::ident)))
        )(s)?;
        let (s, endpoints) = owned_context(
            format!("Parsing {}", name),
            cut(EdgeExp::parse_endpoints),
        )(s)?;
        // Parse the list of fields
        let (s, fields) = owned_context(
            format!("Parsing {}", name),
            cut(Fields::parse),
        )(s)?;

        if let Some((source, _)) = fields.get_field("id".to_string()) {
            return Err(Err::Failure(ParserError::new_at(source, ParserErrorKind::ChangedProtectedField("id".to_string()))))
        }

        Ok((
            s,
            EdgeExp {
                comments,
                attributes,
                name,
                fields,
                endpoints,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for EdgeExp<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        self.comments.compose(f)?;
        self.attributes.compose(f)?;
        write!(f, "edge ")?;
        self.name.compose(f)?;
        write!(f, " ( ")?;
        let mut first = true;
        for endpoint in self.endpoints.values() {
            if !first {
                write!(f, ",")?;
            } else {
                first = false;
            }
            
            endpoint.compose(f)?;
        }
        write!(f, " ) ")?;
        self.fields.compose(f)?;
        Ok(())
    }
}

impl<I> Marked<I> for EdgeExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}


pub(crate) struct EdgeExpOfType<I> {
    pub name: Ident<I>,
    pub node_types: HashSet<String>,
    pub ref_types: HashSet<String>
}

impl<I: Dummy<Faker> + Clone> Dummy<EdgeExpOfType<I>> for EdgeExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &EdgeExpOfType<I>, rng: &mut R) -> Self {
        let mut exp = EdgeExp::dummy_with_rng(&Faker, rng);
        
        // Se the name to the expected value
        exp.name = config.name.clone();
        
        // Make sure all type references point to existing types
        exp.fields = Fields::dummy_with_rng(&FieldWithReferences(config.ref_types.clone()), rng);

        // Update endpoints source and target to align with already created types
        if config.node_types.is_empty() {
            exp.endpoints.clear();
        } else {
            exp.endpoints = exp.endpoints.into_iter().map(|(_, mut endpoint)| {
                endpoint.source = Ident::new(config.node_types.iter().choose(rng).unwrap(), Mark::dummy_with_rng(&Faker, rng));
                endpoint.target = Ident::new(config.node_types.iter().choose(rng).unwrap(), Mark::dummy_with_rng(&Faker, rng));
                ((endpoint.source.clone(), endpoint.target.clone()), endpoint)
            }).collect()
        }

        exp
    }
}

compose_test!{edge_compose, EdgeExp<I>}