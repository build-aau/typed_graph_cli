use build_script_lang::schema::Schema;
use build_script_shared::compose_test;
use build_script_shared::error::ParserSlimResult;
use build_script_shared::InputType;
use fake::Dummy;
use nom::sequence::terminated;
use std::fmt::Display;

use build_script_shared::error::ParserResult;
use build_script_shared::parsers::*;

use nom::branch::*;
use nom::character::complete::char;
use nom::combinator::*;
use nom::error::context;

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
    EditedSchema(EditedSchema),
    EditedFieldType(EditedField<I>),
    EditedOpaque(EditedOpaque<I>),
    EditedType(EditedType<I>),
    EditedVariantsOrder(EditedVariantsOrder<I>),
    EditedEndpoint(EditedEndpoint<I>),
    EditedVariant(EditedVariant<I>),
    EditedGenerics(EditedGenerics<I>),
}

impl<I> SingleChange<I> {
    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> SingleChange<O>
    where
        F: Fn(I) -> O + Copy,
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
            SingleChange::EditedSchema(s) => SingleChange::EditedSchema(s),
            SingleChange::EditedFieldType(s) => SingleChange::EditedFieldType(s.map(f)),
            SingleChange::EditedOpaque(s) => SingleChange::EditedOpaque(s.map(f)),
            SingleChange::EditedType(s) => SingleChange::EditedType(s.map(f)),
            SingleChange::EditedVariantsOrder(s) => SingleChange::EditedVariantsOrder(s.map(f)),
            SingleChange::EditedEndpoint(s) => SingleChange::EditedEndpoint(s.map(f)),
            SingleChange::EditedGenerics(s) => SingleChange::EditedGenerics(s.map(f)),
            SingleChange::EditedVariant(s) => SingleChange::EditedVariant(s.map(f)),
        }
    }

    /// Apply the change to a schema
    pub fn apply(&self, schema: &mut Schema<I>) -> ChangeSetResult<()>
    where
        I: Default + Clone + PartialEq + Ord,
    {
        match self {
            SingleChange::AddedType(s) => s.apply(schema),
            SingleChange::AddedVarient(s) => s.apply(schema),
            SingleChange::AddedField(s) => s.apply(schema),
            SingleChange::AddedEndpoint(s) => s.apply(schema),
            SingleChange::RemovedType(s) => s.apply(schema),
            SingleChange::RemovedVarient(s) => s.apply(schema),
            SingleChange::RemovedField(s) => s.apply(schema),
            SingleChange::EditedSchema(s) => s.apply(schema),
            SingleChange::EditedFieldType(s) => s.apply(schema),
            SingleChange::RemovedEndpoint(s) => s.apply(schema),
            SingleChange::EditedOpaque(s) => s.apply(schema),
            SingleChange::EditedType(s) => s.apply(schema),
            SingleChange::EditedVariantsOrder(s) => s.apply(schema),
            SingleChange::EditedEndpoint(s) => s.apply(schema),
            SingleChange::EditedGenerics(s) => s.apply(schema),
            SingleChange::EditedVariant(s) => s.apply(schema),
        }
    }

    pub fn check_convertion_res(&self) -> ParserSlimResult<I, ()>
    where
        I: Clone,
    {
        match self {
            SingleChange::EditedFieldType(t) => t.check_convertion_res(),
            SingleChange::EditedOpaque(t) => t.check_convertion_res(),
            _ => Ok(()),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for SingleChange<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, change) = context(
            "Parsing SingleChange",
            terminated(
                alt((
                    // Since most of them are very similar the order is super important
                    map(AddedVarient::parse, SingleChange::AddedVarient),
                    map(AddedField::parse, SingleChange::AddedField),
                    map(AddedType::parse, SingleChange::AddedType),
                    map(AddedEndpoint::parse, SingleChange::AddedEndpoint),
                    map(RemovedField::parse, SingleChange::RemovedField),
                    map(RemovedVarient::parse, SingleChange::RemovedVarient),
                    map(RemovedEndpoint::parse, SingleChange::RemovedEndpoint),
                    map(RemovedType::parse, SingleChange::RemovedType),
                    map(EditedSchema::parse, SingleChange::EditedSchema),
                    map(EditedOpaque::parse, SingleChange::EditedOpaque),
                    map(EditedField::parse, SingleChange::EditedFieldType),
                    map(
                        EditedVariantsOrder::parse,
                        SingleChange::EditedVariantsOrder,
                    ),
                    map(EditedGenerics::parse, SingleChange::EditedGenerics),
                    map(EditedVariant::parse, SingleChange::EditedVariant),
                    map(EditedEndpoint::parse, SingleChange::EditedEndpoint),
                    map(EditedType::parse, SingleChange::EditedType),
                )),
                cut(char(';')),
            ),
        )(s)?;

        change.check_convertion_res()?;

        Ok((s, change))
    }
}

impl<I> ParserSerialize for SingleChange<I> {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> build_script_shared::error::ComposerResult<()> {
        match self {
            SingleChange::AddedVarient(s) => s.compose(f, ctx),
            SingleChange::AddedEndpoint(s) => s.compose(f, ctx),
            SingleChange::AddedField(s) => s.compose(f, ctx),
            SingleChange::AddedType(s) => s.compose(f, ctx),
            SingleChange::RemovedEndpoint(s) => s.compose(f, ctx),
            SingleChange::RemovedVarient(s) => s.compose(f, ctx),
            SingleChange::RemovedField(s) => s.compose(f, ctx),
            SingleChange::RemovedType(s) => s.compose(f, ctx),
            SingleChange::EditedSchema(s) => s.compose(f, ctx),
            SingleChange::EditedOpaque(s) => s.compose(f, ctx),
            SingleChange::EditedFieldType(s) => s.compose(f, ctx),
            SingleChange::EditedVariantsOrder(s) => s.compose(f, ctx),
            SingleChange::EditedGenerics(s) => s.compose(f, ctx),
            SingleChange::EditedVariant(s) => s.compose(f, ctx),
            SingleChange::EditedEndpoint(s) => s.compose(f, ctx),
            SingleChange::EditedType(s) => s.compose(f, ctx),
        }?;
        write!(f, ";")?;

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
            SingleChange::EditedSchema(s) => write!(f, "{}", s),
            SingleChange::EditedType(s) => write!(f, "{}", s),
            SingleChange::EditedVariantsOrder(s) => write!(f, "{}", s),
            SingleChange::EditedOpaque(s) => write!(f, "{}", s),
            SingleChange::EditedFieldType(s) => write!(f, "{}", s),
            SingleChange::EditedEndpoint(s) => write!(f, "{}", s),
            SingleChange::EditedGenerics(s) => write!(f, "{}", s),
            SingleChange::EditedVariant(s) => write!(f, "{}", s),
        }
    }
}

compose_test! {single_change_compose, SingleChange<I>}
