use crate::input_marker::InputType;
use crate::error::{ParserError, ParserErrorKind, ParserResult};
use nom::{IResult, Parser, Err};
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::error::{context, ContextError, ParseError};
use nom::multi::*;
use nom::sequence::*;
use nom::character::complete::*;

use super::Mark;

/// Add context to the error stream
/// 
/// This is different from nom::error::context in that it takes a String and node a static String slice
/// 
/// This allows the context to be generated at runtime using the format macro or ToString
pub fn owned_context<I: Clone, F, Context, O>(
    ctx: Context,
    mut f: F,
) -> impl FnMut(I) -> ParserResult<I, O>
where
    F: Parser<I, O, ParserError<I>>,
    Context: ToString,
{
    move |i: I| {
        f.parse(i.clone()).map_err(|mut err| {
            match &mut err {
                Err::Failure(e) | Err::Error(e) => {
                    e.push(i, ParserErrorKind::OwnedContext(ctx.to_string()))
                }
                _ => {}
            };
            err
        })
    }
}


/// Functions the same way as nom::error::append_error but uses ParserErrorKind instead of nom::error::ErrorKind
pub fn append_parser_error<I: Clone, F, O>(mut f: F, kind: ParserErrorKind) -> impl FnMut(I) -> ParserResult<I, O>
where
    F: Parser<I, O, ParserError<I>>,
{
    move |i: I| {
        f.parse(i.clone()).map_err(|mut err| {
            match &mut err {
                Err::Failure(e) | Err::Error(e) => e.push(i, kind.clone()),
                _ => {}
            };
            err
        })
    }
}

/// Parse a u32
/// 
/// This also supports the use om _ anywhere in the middle of the number
pub fn u32<I>(s: I) -> ParserResult<I, u32>
where
    I: InputType,
{
    let (s, (num, marker)) = marked(context(
        "Parsing u32",
        many1(terminated(digit1, many0(char('_')))),
    ))(s)?;

    let num_text: String = num.into_iter().map(|s| s.to_string()).collect();

    let num = match num_text.parse() {
        Ok(num) => num,
        Err(_) => {
            return Err(Err::Failure(ParserError::new_at(
                &marker,
                ParserErrorKind::FailedToParseInteger,
            )));
        }
    };

    Ok((s, num))
}

/// Parse a string literal surrounded by "
pub fn string_data<I>(s: I) -> ParserResult<I, I>
where
    I: InputType,
{
    context(
        "Parsing String literal",
        delimited(char('\"'), take_until("\""), char('\"')),
    )(s)
}

/// Parse a 8-16 character long hex starting with 0x
pub fn hex_u64<I>(s: I) -> ParserResult<I, u64>
where
    I: InputType,
{
    let (s, res) = preceded(
        pair(char('0'), char('x')),
        fold_many_m_n(8, 16, satisfy(|c| c.is_digit(16)), move || Vec::new(), |mut acc, c| {
            acc.push(c.to_digit(16).unwrap_or_default() as u64);
            acc
        })
    )(s)?;

    let mut total: u64 = 0;
    for i in 0..res.len() {
        total += res[res.len() - 1 - i] << i*4;
    }

    Ok((s, total))
}

/// Remove all whitespaces and tabs around a token
pub fn ws<I, F, O, E>(inner: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputType,
    E: ParseError<I> + ContextError<I>,
    F: Parser<I, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Get the marker for the parsed token
pub fn marked<I, F, O, E>(mut inner: F) -> impl FnMut(I) -> IResult<I, (O, Mark<I>), E>
where
    I: InputType,
    E: ParseError<I> + ContextError<I>,
    F: Parser<I, O, E>,
{
    move |s: I| {
        let marker_start = s.clone();
        let (s, v) = inner.parse(s)?;
        let marker = marker_start.slice(..marker_start.offset(&s));
        Ok((s, (v, Mark::new(marker))))
    }
}

/// Parse a key value pair with a custom seperator inbetween
/// 
/// This is very much akin to separated_pair 
/// with the one difference that if the key and seperator is parsed 
/// then the value is expected to be there 
/// otherwise the parser will fail
pub fn key_value<I, O1, O2, O3, Sep, K, V>(
    key: K,
    seperator: Sep,
    value: V,
) -> impl FnMut(I) -> ParserResult<I, (O1, O3)>
where
    I: InputType,
    K: Parser<I, O1, ParserError<I>>,
    Sep: Parser<I, O2, ParserError<I>>,
    V: Parser<I, O3, ParserError<I>>,
{
    context(
        "Parsing KeyValue", 
        separated_pair(
            key,
            ws(seperator),
            cut(context("Expected value", value)),
        )
    )
}

/// Parse a token surrounded by specific chars
///
/// This is most commonly used for (), {} or []
pub fn surrounded<I, O, E, P>(
    before: char,
    p: P,
    after: char,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputType,
    E: ParseError<I> + ContextError<I>,
    P: Parser<I, O, E>,
{
    context("Parsing Surround", 
        delimited(
            ws(char(before)), 
            p, 
            ws(context("Expected closing character", cut(
                char(after)
            )))
        )
    )
}

/// Parse a punctuated list of tokens
pub fn punctuated<I, O, E, P>(
    mut p: P,
    seperator: char,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: InputType,
    E: ParseError<I> + ContextError<I>,
    P: FnMut(I) -> IResult<I, O, E>,
{
    move |s| {
        // First parse one object
        let (s, head) = context("Parsing Punctuated", opt(ws(&mut p)))(s)?;
        // Then parse every following object
        let (s, mut tail) = context("Parsing Punctuated", many0(preceded(ws(char(seperator)), &mut p)))(s)?;
        // Remove trailing comma
        let (s, _) = opt(ws(char(seperator)))(s)?;

        if let Some(head) = head {
            // add head object to beginning of all the tail arguments
            tail.insert(0, head);
        }

        Ok((s, tail))
    }
}


#[test]
fn string_test() {
    assert_eq!(
        string_data("\"aa\""),
        ParserResult::<&str, _>::Ok(("", "aa"))
    );
    assert_eq!(
        string_data("\"Hello World 1234\""),
        ParserResult::<&str, _>::Ok(("", "Hello World 1234"))
    );
}

#[test]
fn ws_test() {
    assert_eq!(
        ws(tag("aa"))("       aa        "),
        IResult::<&str, _>::Ok(("", "aa"))
    );
}

#[test]
fn key_value_test() {
    assert_eq!(
        key_value(tag("aa"), char(':'), tag("bb"))("aa:bb"),
        ParserResult::<&str, _>::Ok(("", ("aa", "bb")))
    );
}

#[test]
fn surrounded_test() {
    assert_eq!(
        surrounded('(', tag("aa"), ')')("(aa)"),
        IResult::<&str, _>::Ok(("", "aa"))
    );
    assert_eq!(
        surrounded('(', tag("aa"), ')')("(   aa   )"),
        IResult::<&str, _>::Ok(("", "aa"))
    );
    assert_eq!(
        surrounded('<', tag("aa"), '>')("<aa>"),
        IResult::<&str, _>::Ok(("", "aa"))
    );
    assert_eq!(
        surrounded('(', tag("aa"), ')')("(aa)asdasd"),
        IResult::<&str, _>::Ok(("asdasd", "aa"))
    );
}

#[test]
fn punctuated_test() {
    assert_eq!(
        punctuated(tag("a"), ',')("a,a,a,a,a"),
        IResult::<&str, _>::Ok(("", vec!["a", "a", "a", "a", "a"]))
    );
    assert_eq!(
        punctuated(tag("a"), ',')("a     , a,     a,  a,a"),
        IResult::<&str, _>::Ok(("", vec!["a", "a", "a", "a", "a"]))
    );
    assert_eq!(
        punctuated(tag("a"), ',')("a,a,a,a,a,"),
        IResult::<&str, _>::Ok(("", vec!["a", "a", "a", "a", "a"]))
    );
    assert_eq!(
        punctuated(tag("a"), ',')("aa,a,a,a"),
        IResult::<&str, _>::Ok(("a,a,a,a", vec!["a"]))
    );
    assert_eq!(
        punctuated(tag("a"), ',')("a,,a,a,a,a,"),
        IResult::<&str, _>::Ok((",a,a,a,a,", vec!["a"]))
    );
}
