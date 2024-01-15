use super::*;
use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use fake::*;
use nom::branch::*;
use nom::bytes::complete::*;
use nom::character::complete::char;
use nom::combinator::*;
use nom::error::context;
use nom::sequence::{terminated, preceded};
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Debug, Clone, Hash, PartialOrd, Ord, Dummy)]
pub enum Types<I> {
    String(Mark<I>),
    Bool(Mark<I>),
    F64(Mark<I>),
    F32(Mark<I>),
    Usize(Mark<I>),
    U64(Mark<I>),
    U32(Mark<I>),
    U16(Mark<I>),
    U8(Mark<I>),
    Isize(Mark<I>),
    I64(Mark<I>),
    I32(Mark<I>),
    I16(Mark<I>),
    I8(Mark<I>),
    Option(Box<Types<I>>, Mark<I>),
    List(Box<Types<I>>, Mark<I>),
    Map(Box<Types<I>>, Box<Types<I>>, Mark<I>),
    Reference(#[dummy(faker = "SimpleIdentDummy")] Ident<I>),
}

impl<I> Types<I> {
    pub fn is_valid(&self, all_reference_types: &HashSet<Ident<I>>) -> Result<(), &Types<I>> {
        match self {
            Types::String(_)
            | Types::Usize(_)
            | Types::Bool(_)
            | Types::F64(_)
            | Types::F32(_)
            | Types::U64(_)
            | Types::U32(_)
            | Types::U16(_)
            | Types::U8(_)
            | Types::Isize(_)
            | Types::I64(_)
            | Types::I32(_)
            | Types::I16(_)
            | Types::I8(_) => Ok(()),
            Types::Option(ty, _) => ty.is_valid(all_reference_types),
            Types::List(ty, _) => ty.is_valid(all_reference_types),
            Types::Map(kty, vty, _) => kty.is_valid(all_reference_types).and_then(|_| vty.is_valid(all_reference_types)) ,
            Types::Reference(ty) => if all_reference_types.contains(ty) {
                Ok(())
            } else {
                Err(self)
            },
        }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Types<O>
    where
        F: FnMut(I) -> O + Copy,
    {
        match self {
            Types::String(s) => Types::String(s.map(f)),
            Types::Bool(ty) => Types::Bool(ty.map(f)),
            Types::F64(ty) => Types::F64(ty.map(f)),
            Types::F32(ty) => Types::F32(ty.map(f)),
            Types::Usize(s) => Types::Usize(s.map(f)),
            Types::U64(s) => Types::U64(s.map(f)),
            Types::U32(s) => Types::U32(s.map(f)),
            Types::U16(s) => Types::U16(s.map(f)),
            Types::U8(s) => Types::U8(s.map(f)),
            Types::Isize(s) => Types::Isize(s.map(f)),
            Types::I64(s) => Types::I64(s.map(f)),
            Types::I32(s) => Types::I32(s.map(f)),
            Types::I16(s) => Types::I16(s.map(f)),
            Types::I8(s) => Types::I8(s.map(f)),
            Types::List(ty, s) => Types::List(ty.map(f).into(), s.map(f)),
            Types::Option(ty, s) => Types::Option(ty.map(f).into(), s.map(f)),
            Types::Map(kty, vty, s) => Types::Map(kty.map(f).into(), vty.map(f).into(), s.map(f)),
            Types::Reference(ty) => Types::Reference(ty.map(f)),
        }
    }
}

impl<I: InputType> ParserDeserialize<I> for Types<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, ty) = context(
            "Parsing Types",
            alt((
                map(
                    marked(terminated(
                        preceded(
                            tag("Option"), 
                            surrounded('<', cut(Types::parse), '>')
                        ), 
                        not(Ident::ident_full)
                    )),
                    |(ty, marker)| Types::Option(ty.into(), marker),
                ),
                map(
                    marked(terminated(
                        preceded(
                            tag("List"), 
                            surrounded('<', cut(Types::parse), '>')
                        ), 
                        not(Ident::ident_full)
                    )),
                    |(ty, marker)| Types::List(ty.into(), marker),
                ),
                map(
                    marked(terminated(
                        preceded(
                            tag("Map"), 
                            surrounded('<', cut(key_value(Types::parse, char(','), Types::parse)), '>')
                        ), 
                        not(Ident::ident_full)
                    )),
                    |((kty, vty), marker)| Types::Map(kty.into(), vty.into(), marker),
                ),
                map(
                    marked(terminated(tag("String"), not(Ident::ident_full))),
                    |(_, marker)| Types::String(marker),
                ),
                map(
                    marked(terminated(tag("bool"), not(Ident::ident_full))),
                    |(_, marker)| Types::Bool(marker),
                ),
                map(
                    marked(terminated(tag("f64"), not(Ident::ident_full))),
                    |(_, marker)| Types::F64(marker),
                ),
                map(
                    marked(terminated(tag("f32"), not(Ident::ident_full))),
                    |(_, marker)| Types::F32(marker),
                ),
                map(
                    marked(terminated(tag("usize"), not(Ident::ident_full))),
                    |(_, marker)| Types::Usize(marker),
                ),
                map(
                    marked(terminated(tag("u64"), not(Ident::ident_full))),
                    |(_, marker)| Types::U64(marker),
                ),
                map(
                    marked(terminated(tag("u32"), not(Ident::ident_full))),
                    |(_, marker)| Types::U32(marker),
                ),
                map(
                    marked(terminated(tag("u16"), not(Ident::ident_full))),
                    |(_, marker)| Types::U16(marker),
                ),
                map(
                    marked(terminated(tag("u8"), not(Ident::ident_full))),
                    |(_, marker)| Types::U8(marker),
                ),
                map(
                    marked(terminated(tag("isize"), not(Ident::ident_full))),
                    |(_, marker)| Types::Isize(marker),
                ),
                map(
                    marked(terminated(tag("i64"), not(Ident::ident_full))),
                    |(_, marker)| Types::I64(marker),
                ),
                map(
                    marked(terminated(tag("i32"), not(Ident::ident_full))),
                    |(_, marker)| Types::I32(marker),
                ),
                map(
                    marked(terminated(tag("i16"), not(Ident::ident_full))),
                    |(_, marker)| Types::I16(marker),
                ),
                map(
                    marked(terminated(tag("i8"), not(Ident::ident_full))),
                    |(_, marker)| Types::I8(marker),
                ),
                map(Ident::ident, |ty| Types::Reference(ty)),
            )),
        )(s)?;

        Ok((s, ty))
    }
}

impl<I> ParserSerialize for Types<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        match self {
            Types::String(_) => write!(f, "String")?,
            Types::Bool(_) => write!(f, "bool")?,
            Types::F64(_) => write!(f, "f64")?,
            Types::F32(_) => write!(f, "f32")?,
            Types::Usize(_) => write!(f, "usize")?,
            Types::U64(_) => write!(f, "u64")?,
            Types::U32(_) => write!(f, "u32")?,
            Types::U16(_) => write!(f, "u16")?,
            Types::U8(_) => write!(f, "u8")?,
            Types::Isize(_) => write!(f, "isize")?,
            Types::I64(_) => write!(f, "i64")?,
            Types::I32(_) => write!(f, "i32")?,
            Types::I16(_) => write!(f, "i16")?,
            Types::I8(_) => write!(f, "i8")?,
            Types::Option(ty, _) => write!(f, "Option<{ty}>")?,
            Types::List(ty, _) => write!(f, "List<{ty}>")?,
            Types::Map(kty, vty, _) => write!(f, "Map<{kty}, {vty}>")?,
            Types::Reference(r) => r.compose(f)?,
        };
        Ok(())
    }
}

#[test]
fn type_test() {
    assert_eq!(
        Types::parse("String"),
        Ok(("", Types::String(Mark::new("String"))))
    );
    assert_eq!(
        Types::parse("bool"),
        Ok(("", Types::Bool(Mark::new("bool"))))
    );
    assert_eq!(Types::parse("f64"), Ok(("", Types::F64(Mark::new("f64")))));
    assert_eq!(Types::parse("f32"), Ok(("", Types::F32(Mark::new("f32")))));
    assert_eq!(
        Types::parse("usize"),
        Ok(("", Types::Usize(Mark::new("String"))))
    );
    assert_eq!(Types::parse("u8"), Ok(("", Types::U8(Mark::new("String")))));
    assert_eq!(
        Types::parse("as1d4f33sda1"),
        Ok((
            "",
            Types::Reference(Ident::new("as1d4f33sda1", Mark::new("as1d4f33sda1")))
        ))
    );
    assert_eq!(
        Types::parse("asdfsda:asa"),
        Ok((
            ":asa",
            Types::Reference(Ident::new("asdfsda", Mark::new("asdfsda")))
        ))
    );
}

impl<I> PartialEq for Types<I> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Types::String(_), Types::String(_))
            | (Types::Bool(_), Types::Bool(_))
            | (Types::F64(_), Types::F64(_))
            | (Types::F32(_), Types::F32(_))
            | (Types::Usize(_), Types::Usize(_))
            | (Types::U64(_), Types::U64(_))
            | (Types::U32(_), Types::U32(_))
            | (Types::U16(_), Types::U16(_))
            | (Types::U8(_), Types::U8(_))
            | (Types::Isize(_), Types::Isize(_))
            | (Types::I64(_), Types::I64(_))
            | (Types::I32(_), Types::I32(_))
            | (Types::I16(_), Types::I16(_))
            | (Types::I8(_), Types::I8(_)) => true,
            (Types::List(type0, _), Types::List(type1, _)) => type0.eq(type1),
            (Types::Option(type0, _), Types::Option(type1, _)) => type0.eq(type1),
            (Types::Map(ktype0, vtype0, _), Types::Map(ktype1, vtype1, _)) => ktype0.eq(ktype1) && vtype0.eq(vtype1),
            (Types::Reference(type0), Types::Reference(type1)) => type0.eq(type1),
            _ => false,
        }
    }
}

impl<I> Eq for Types<I> {}

impl<I> Display for Types<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::String(_) => write!(f, "String"),
            Types::Bool(_) => write!(f, "bool"),
            Types::F64(_) => write!(f, "f64"),
            Types::F32(_) => write!(f, "f32"),
            Types::Usize(_) => write!(f, "usize"),
            Types::U64(_) => write!(f, "u64"),
            Types::U32(_) => write!(f, "u32"),
            Types::U16(_) => write!(f, "u16"),
            Types::U8(_) => write!(f, "u8"),
            Types::Isize(_) => write!(f, "isize"),
            Types::I64(_) => write!(f, "i64"),
            Types::I32(_) => write!(f, "i32"),
            Types::I16(_) => write!(f, "i16"),
            Types::I8(_) => write!(f, "i8"),
            Types::Option(ty, _) => write!(f, "Option<{ty}>"),
            Types::List(ty, _) => write!(f, "List<{ty}>"),
            Types::Map(kty, vty, _) => write!(f, "Map<{kty}, {vty}>"),
            Types::Reference(name) => name.fmt(f),
        }
    }
}

impl<I> Marked<I> for Types<I> {
    fn marker(&self) -> &Mark<I> {
        match self {
            Types::String(marker) => &marker,
            Types::Bool(marker) => &marker,
            Types::F64(marker) => &marker,
            Types::F32(marker) => &marker,
            Types::Usize(marker) => &marker,
            Types::U64(marker) => &marker,
            Types::U32(marker) => &marker,
            Types::U16(marker) => &marker,
            Types::U8(marker) => &marker,
            Types::Isize(marker) => &marker,
            Types::I64(marker) => &marker,
            Types::I32(marker) => &marker,
            Types::I16(marker) => &marker,
            Types::I8(marker) => &marker,
            Types::Option(_, marker) => &marker,
            Types::List(_, marker) => &marker,
            Types::Map(_, _, marker) => &marker,
            Types::Reference(name) => name.marker(),
        }
    }
}

compose_test! {types_compose, Types<I>}
