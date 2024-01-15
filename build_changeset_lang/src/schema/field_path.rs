use build_script_lang::schema::*;
use build_script_shared::compose_test;
use build_script_shared::error::ParserResult;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use fake::Dummy;
use nom::character::complete::char;
use nom::error::context;
use nom::sequence::{pair, preceded};
use nom::combinator::cut;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Dummy)]
pub struct FieldPath<I> {
    pub root: Ident<I>,
    pub path: Vec<Ident<I>>,
}

impl<I> FieldPath<I> {
    pub fn new(root: Ident<I>) -> FieldPath<I> {
        FieldPath {
            root,
            path: Vec::new(),
        }
    }

    pub fn new_path(root: Ident<I>, path: Vec<Ident<I>>) -> FieldPath<I> {
        FieldPath {
            root,
            path
        }
    }

    pub fn retrieve_fields<'a>(&'a self, schema: &'a mut Schema<I>) -> Option<(&'a mut Fields<I>, &'a Ident<I>)> {
        schema
            .content
            .iter_mut()
            .find(|s| s.get_type() == &self.root)?
            .get_fields_mut()
            .zip(Some(&self.path[0]))
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> FieldPath<O> 
    where
        F: Fn(I) -> O + Copy
    {
        FieldPath {
            root: self.root.map(f),
            path: self.path.into_iter().map(|p| p.map(f)).collect()
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for FieldPath<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, (root, path)) = context(
            "Parsing FieldPath",
            pair(
                Ident::ident, 
                preceded(
                    ws(char('.')), 
                    cut(punctuated(Ident::ident, '.'))
                )
            )
        )(s)?;

        Ok((s, FieldPath { root, path }))
    }
}

impl<I> ParserSerialize for FieldPath<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        self.root.compose(f)?;
        write!(f, ".")?;
        let iter = self.path.iter().enumerate();
        for (i, seg) in iter {
            seg.compose(f)?;
            if i + 1 != self.path.len() {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

impl<I> Display for FieldPath<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)?;
        for seg in &self.path {
            write!(f, ".{}", seg)?;
        }

        Ok(())
    }
}

impl<I> FieldPath<I>
where
    I: Clone,
{
    pub(crate) fn push(&self, head: Ident<I>) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(head);
        FieldPath {
            root: self.root.clone(),
            path: new_path,
        }
    }
}

compose_test!{field_path_compose, FieldPath<I>}