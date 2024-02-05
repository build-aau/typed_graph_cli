use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use build_script_shared::dependency_graph::DependencyGraph;
use build_script_shared::error::{ParserError, ParserErrorKind, ParserSlimResult};
use build_script_shared::parsers::{marked, punctuated, surrounded, ws, Comments, ComposeContext, Ident, Mark, Marked, ParserDeserialize, ParserSerialize, TypeReferenceMap, Types};
use build_script_shared::{compose_test, InputType};
use fake::{Dummy, Faker};
use nom::combinator::{cut, success};
use nom::error::context;
use nom::Err;
use serde::{Serialize, Deserialize};
use super::{FieldWithReferences, Fields};
use nom::branch::alt;
use nom::combinator::map;

#[derive(PartialEq, Eq, Hash, Debug, Clone, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "I: Default + Clone")]
pub enum EnumVarient<I> {
    // Quick note: I did implement Tuple enum and they worked
    // However there was no good way of making changesets.
    // if people made a new schema with a new field in the middle of the tuple
    // then i basicly invalidaded all the fields after it as there was no way of knowing 
    // where to retrieve the previous versions fields from
    Struct {
        name: Ident<I>,
        comments: Comments,
        fields: Fields<I>,
        #[serde(skip)]
        marker: Mark<I>
    },
    Unit {
        name: Ident<I>,
        comments: Comments,
        #[serde(skip)]
        marker: Mark<I>
    }
}

impl<I> EnumVarient<I> {
    pub fn name(&self) -> &Ident<I> {
        match self {
            EnumVarient::Struct { name, .. } => &name,
            EnumVarient::Unit { name, .. } => &name,
        }
    }

    pub fn comments(&self) -> &Comments {
        match self {
            EnumVarient::Struct { comments, .. } => comments,
            EnumVarient::Unit { comments, .. } => comments,
        }
    }

    pub fn comments_mut(&mut self) -> &mut Comments {
        match self {
            EnumVarient::Struct { comments, .. } => comments,
            EnumVarient::Unit { comments, .. } => comments,
        }
    }

    pub fn strip_comments(&mut self) {
        match self {
            EnumVarient::Struct { comments, fields, .. } => {
                comments.strip_comments();
                fields.strip_comments();
            },
            EnumVarient::Unit { comments, .. } => {
                comments.strip_comments();
            }
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> EnumVarient<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            EnumVarient::Struct { 
                name,
                comments, 
                fields,
                marker
            } => EnumVarient::Struct { 
                name: name.map(f),
                comments, 
                fields: fields.map(f),
                marker: marker.map(f),
            },
            EnumVarient::Unit { 
                name,
                comments,
                marker
            } => EnumVarient::Unit { 
                name: name.map(f),
                comments,
                marker: marker.map(f)
            }
        }
    }

    pub fn check_types(
        &self,
        reference_types: &HashMap<Ident<I>, Vec<String>>
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct { fields, .. } => {
                fields.check_types(reference_types)?
            },
            EnumVarient::Unit { .. } => ()
        };

        Ok(())
    }

    pub fn check_cycle<'a>(
        &'a self,
        type_name: &'a Ident<I>,
        type_generics: &Vec<String>,
        dependency_graph: &mut DependencyGraph<'a, I>,
    ) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct { fields, .. } => fields.check_cycle(type_name, type_generics, dependency_graph)?,
            EnumVarient::Unit { .. } => ()
        };
        Ok(())
    }

    pub fn remove_used(&self, reference_types: &mut HashSet<Ident<I>>)
    where
        I: Clone,
    {
        match self {
            EnumVarient::Struct { fields, .. } => fields.remove_used(reference_types),
            EnumVarient::Unit { .. } => ()
        }
    }
}

impl<I> Marked<I> for EnumVarient<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            EnumVarient::Struct { marker, .. } => marker,
            EnumVarient::Unit { marker, .. } => marker,
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for EnumVarient<I> {
    fn parse(s: I) -> build_script_shared::error::ParserResult<I, Self> {
        let (s, comments) = Comments::parse(s)?;
        let (s, (name, marker)) = context("Parsing Enum varient", ws(marked(Ident::ident)))(s)?;

        let (s, varient) = alt((
            map(
                context("Parsing Struct enum", Fields::parse), 
                // Its not the prettiest thing that we clone here but it will do
                |fields| EnumVarient::Struct { 
                    name: name.clone(), 
                    comments: comments.clone(), 
                    fields,
                    marker: marker.clone()
                }
            ),
            success(EnumVarient::Unit { 
                name: name.clone(), 
                comments: comments.clone(), 
                marker: marker.clone()
            })
        ))(s)?;

        Ok((s, varient))
    }
}

impl<I> ParserSerialize for EnumVarient<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W, ctx: ComposeContext) -> build_script_shared::error::ComposerResult<()> {
        match self {
            EnumVarient::Struct { 
                name,
                comments,
                fields,
                ..
            } => {
                comments.compose(f, ctx)?;
                name.compose(f, ctx)?;
                write!(f, " ")?;
                fields.compose(f, ctx)?;
            }
            EnumVarient::Unit { 
                name, 
                comments,
                ..
            } => {
                comments.compose(f, ctx)?;
                name.compose(f, ctx)?;
            }
        }

        Ok(())
    }
}

pub struct EnumVarientOfType {
    pub name: String,
    pub ref_types: TypeReferenceMap,
    
}

impl<I: Dummy<Faker> + Clone> Dummy<EnumVarientOfType> for EnumVarient<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &EnumVarientOfType, rng: &mut R) -> Self {
        let mut varient = EnumVarient::dummy_with_rng(&Faker, rng);

        match &mut varient {
            EnumVarient::Struct { name, .. } => *name = Ident::new(config.name.clone(), Mark::dummy_with_rng(&Faker, rng)),
            EnumVarient::Unit { name, .. } => *name = Ident::new(config.name.clone(), Mark::dummy_with_rng(&Faker, rng)),
        };

        match &mut varient {
            EnumVarient::Struct { fields, .. } => {
                *fields = Fields::dummy_with_rng(&FieldWithReferences(config.ref_types.clone()), rng);
            },
            EnumVarient::Unit { .. } => ()
        }

        varient
    }
}

impl<I: Dummy<Faker>> Dummy<Faker> for EnumVarient<I> {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let varient = rng.gen_range(0..2);

        match varient {
            0 => {
                let fields = Fields::dummy_with_rng(config, rng);

                if !fields.is_empty() {
                    EnumVarient::Struct { 
                        name: Dummy::dummy_with_rng(config, rng), 
                        comments: Dummy::dummy_with_rng(config, rng), 
                        fields,
                        marker: Dummy::dummy_with_rng(config, rng), 
                    }
                } else {
                    EnumVarient::Unit { 
                        name: Dummy::dummy_with_rng(config, rng), 
                        comments: Dummy::dummy_with_rng(config, rng), 
                        marker: Dummy::dummy_with_rng(config, rng), 
                    }
                }
            },
            _ => EnumVarient::Unit { 
                name: Dummy::dummy_with_rng(config, rng), 
                comments: Dummy::dummy_with_rng(config, rng), 
                marker: Dummy::dummy_with_rng(config, rng), 
            },
        }
    }
}

compose_test! {enum_varient_compose, EnumVarient<I>}

