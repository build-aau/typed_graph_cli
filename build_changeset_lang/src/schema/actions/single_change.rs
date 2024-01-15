use build_script_lang::schema::Schema;
use build_script_shared::InputType;
use build_script_shared::compose_test;
use fake::Dummy;
use nom::sequence::terminated;
use std::fmt::Display;

use build_script_shared::error::ParserResult;
use build_script_shared::parsers::*;

use nom::error::context;
use nom::branch::*;
use nom::combinator::*;
use nom::character::complete::char;

use crate::*;

/// Represent a single change in the schema
/// 
/// Changes are seperated into add, edit and remove events
/// 
/// This makes it easier to see what actually changed
#[derive(PartialEq, Eq, Debug, Clone, Hash, Dummy)]
pub enum SingleChange<I> {
    AddedType(AddedType<I>),
    AddedVarient(AddedVarient<I>),
    AddedField(AddedField<I>),
    AddedEndpoint(AddedEndpoint<I>),
    RemovedType(RemovedType<I>),
    RemovedVarient(RemovedVarient<I>),
    RemovedEndpoint(RemovedEndpoint<I>),
    RemovedField(RemovedField<I>),
    EditedFieldType(EditedField<I>),
    EditedType(EditedType<I>),
    EditedEndpoint(EditedEndpoint<I>),
    
}

impl<I> SingleChange<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> SingleChange<O> 
    where
        F: Fn(I) -> O + Copy
    {
        match self {
            SingleChange::AddedType(s) => SingleChange::AddedType(s.map(f)),
            SingleChange::AddedVarient(s) => SingleChange::AddedVarient(s.map(f)),
            SingleChange::AddedField(s) => SingleChange::AddedField(s.map(f)),
            SingleChange::AddedEndpoint(s) => SingleChange::AddedEndpoint(s.map(f)),
            SingleChange::RemovedType(s) => SingleChange::RemovedType(s.map(f)),
            SingleChange::RemovedVarient(s) => SingleChange::RemovedVarient(s.map(f)),
            SingleChange::RemovedField(s) => SingleChange::RemovedField(s.map(f)),
            SingleChange::RemovedEndpoint(s) => SingleChange::RemovedEndpoint(s.map(f)),
            SingleChange::EditedFieldType(s) => SingleChange::EditedFieldType(s.map(f)),
            SingleChange::EditedType(s) => SingleChange::EditedType(s.map(f)),
            SingleChange::EditedEndpoint(s) => SingleChange::EditedEndpoint(s.map(f)),
        }
    }

    /// Apply the change to a schema
    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()> 
    where
        I: Default + Clone + PartialEq
    {
        match self {
            SingleChange::AddedType(s) => s.apply(schema),
            SingleChange::AddedVarient(s) => s.apply(schema),
            SingleChange::AddedField(s) => s.apply(schema),
            SingleChange::AddedEndpoint(s) => s.apply(schema),
            SingleChange::RemovedType(s) => s.apply(schema),
            SingleChange::RemovedVarient(s) => s.apply(schema),
            SingleChange::RemovedField(s) => s.apply(schema),
            SingleChange::RemovedEndpoint(s) => s.apply(schema),
            SingleChange::EditedFieldType(s) => s.apply(schema),
            SingleChange::EditedType(s) => s.apply(schema),
            SingleChange::EditedEndpoint(s) => s.apply(schema),

        }
    }
}

impl<I: InputType> ParserDeserialize<I> for SingleChange<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let res = context("Parsing SingleChange", alt((
            terminated(map(AddedType::parse, SingleChange::AddedType), ws(char(';'))),
            terminated(map(AddedVarient::parse, SingleChange::AddedVarient), ws(char(';'))),
            terminated(map(AddedField::parse, SingleChange::AddedField), ws(char(';'))),
            terminated(map(AddedEndpoint::parse, SingleChange::AddedEndpoint), ws(char(';'))),
            terminated(map(RemovedType::parse, SingleChange::RemovedType), ws(char(';'))),
            terminated(map(RemovedField::parse, SingleChange::RemovedField), ws(char(';'))),
            terminated(map(RemovedVarient::parse, SingleChange::RemovedVarient), ws(char(';'))),
            terminated(map(RemovedEndpoint::parse, SingleChange::RemovedEndpoint), ws(char(';'))),
            terminated(map(EditedField::parse, SingleChange::EditedFieldType), ws(char(';'))),
            terminated(map(EditedType::parse, SingleChange::EditedType), ws(char(';'))),
            terminated(map(EditedEndpoint::parse, SingleChange::EditedEndpoint), ws(char(';'))),
            fail
        )))(s)?;

        Ok(res)
    }
}

impl<I> ParserSerialize for SingleChange<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> build_script_shared::error::ComposerResult<()> {
        match self {
            SingleChange::AddedType(s) => s.compose(f),
            SingleChange::AddedVarient(s) => s.compose(f),
            SingleChange::AddedField(s) => s.compose(f),
            SingleChange::AddedEndpoint(s) => s.compose(f),
            SingleChange::RemovedType(s) => s.compose(f),
            SingleChange::RemovedVarient(s) => s.compose(f),
            SingleChange::RemovedField(s) => s.compose(f),
            SingleChange::RemovedEndpoint(s) => s.compose(f),
            SingleChange::EditedFieldType(s) => s.compose(f),
            SingleChange::EditedType(s) => s.compose(f),
            SingleChange::EditedEndpoint(s) => s.compose(f),
        }?;
        writeln!(f, ";")?;

        Ok(())
    }
}

impl<I> Display for SingleChange<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleChange::AddedType(s) => write!(f, "{}", s),
            SingleChange::AddedVarient(s) => write!(f, "{}", s),
            SingleChange::AddedField(s) => write!(f, "{}", s),
            SingleChange::AddedEndpoint(s) => write!(f, "{}", s),
            SingleChange::RemovedType(s) => write!(f, "{}", s),
            SingleChange::RemovedVarient(s) => write!(f, "{}", s),
            SingleChange::RemovedField(s) => write!(f, "{}", s),
            SingleChange::RemovedEndpoint(s) => write!(f, "{}", s),
            SingleChange::EditedType(s) => write!(f, "{}", s),
            SingleChange::EditedFieldType(s) => write!(f, "{}", s),
            SingleChange::EditedEndpoint(s) => write!(f, "{}", s),
        }
    }
}

compose_test!{single_change_compose, SingleChange<I>}