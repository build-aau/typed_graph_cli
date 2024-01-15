use std::collections::HashSet;

use super::FieldWithReferences;
use super::Fields;
use build_script_shared::compose_test;
use build_script_shared::error::ParserError;
use build_script_shared::error::ParserErrorKind;
use build_script_shared::error::ParserResult;
use build_script_shared::error::ParserSlimResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use fake::Faker;
use nom::Err;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::*;
use nom::multi::*;
use nom::sequence::*;
use rand::seq::IteratorRandom;

const RENAME_INC: &str = "rename_inc";
const RENAME_OUT: &str = "rename_out";

const ALLOWED_ATTRIBUTES: &[(&str, usize)] = &[
    (RENAME_INC, 2), 
    (RENAME_OUT, 2)
];

#[derive(PartialEq, Eq, Debug, Hash, Clone, Default, PartialOrd, Ord, Dummy)]
pub struct NodeExp<I> {
    pub name: Ident<I>,
    #[dummy(faker = "AllowedFunctionAttribute(ALLOWED_ATTRIBUTES)")]
    pub attributes: Attributes<I>,
    pub comments: Comments,
    pub fields: Fields<I>,
    marker: Mark<I>,
}

impl<I> NodeExp<I> {
    pub fn new(comments: Comments, attributes: Attributes<I>, name: Ident<I>, fields: Fields<I>, marker: Mark<I>) -> Self {
        NodeExp {
            comments,
            attributes,
            name,
            fields,
            marker,
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> NodeExp<O> 
    where
        F: FnMut(I) -> O + Copy,
    {
        NodeExp {
            comments: self.comments,
            attributes: self.attributes.map(f),
            name: self.name.map(f),
            fields: self.fields.map(f),
            marker: self.marker.map(f)
        }
    }

    pub fn check_attributes(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        self.attributes.check_function(ALLOWED_ATTRIBUTES)?;
        Ok(())
    }

    pub fn check_types(&self, node_reference_types: &HashSet<Ident<I>>) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        let rename_out = self.attributes.get_functions(RENAME_OUT);
        for function in rename_out {
            let target_node = function
                .values
                .get(&1)
                .ok_or_else(|| Err::Failure(ParserError::new_at(
                    function,
                    ParserErrorKind::MissingRequiredField("target_node".to_string()),
                )))?;
            
            if !node_reference_types.contains(target_node) {
                return Err(Err::Failure(ParserError::new_at(
                    function,
                    ParserErrorKind::UnknownReference(target_node.to_string())
                )));
            }
        }

        let rename_in = self.attributes.get_functions(RENAME_INC);
        for function in rename_in {
            let source_node = function
                .values
                .get(&1)
                .ok_or_else(|| Err::Failure(ParserError::new_at(
                    function,
                    ParserErrorKind::MissingRequiredField("source_node".to_string()),
                )))?;
            
            if !node_reference_types.contains(source_node) {
                return Err(Err::Failure(ParserError::new_at(
                    function,
                    ParserErrorKind::UnknownReference(source_node.to_string())
                )));
            }
        }
        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for NodeExp<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, attributes) = Attributes::parse(s)?;
        // Parse the name
        let (s, _) = ws(terminated(tag("node"), many1(multispace1)))(s)?;
        // Parse the name
        let (s, (name, marker)) = context(
            "Parsing Node type", 
            ws(cut(marked(Ident::ident)))
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
            NodeExp {
                comments,
                attributes,
                name,
                fields,
                marker,
            },
        ))
    }
}

impl<I> ParserSerialize for NodeExp<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.comments.compose(f)?;
        self.attributes.compose(f)?;
        write!(f, "node ")?;
        self.name.compose(f)?;
        write!(f, " ")?;
        self.fields.compose(f)?;

        Ok(())
    }
}

impl<I> Marked<I> for NodeExp<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

pub(crate) struct NodeExpOfType<I> {
    pub name: Ident<I>,
    pub ref_types: HashSet<String>,
    pub node_types: HashSet<String>
}

impl<I: Dummy<Faker> + Clone> Dummy<NodeExpOfType<I>> for NodeExp<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &NodeExpOfType<I>, rng: &mut R) -> Self {
        let mut exp = NodeExp::dummy_with_rng(&Faker, rng);
        
        // Set the name to the expected value
        exp.name = config.name.clone();

        // Make sure all type references point to existing types
        exp.fields = Fields::dummy_with_rng(&FieldWithReferences(config.ref_types.clone()), rng);

        for attr in exp.attributes.attributes.values_mut() {
            if let Attribute::Function(f) = attr {
                if RENAME_INC == *f.key || RENAME_OUT == *f.key {
                    let node_type = f.values.get_mut(&1).unwrap();
                    *node_type = Ident::new(config.node_types.iter().choose(rng).unwrap(), Mark::dummy(&Faker));
                }
            }
        }

        exp
    }
}

compose_test!{node_compose, NodeExp<I>}