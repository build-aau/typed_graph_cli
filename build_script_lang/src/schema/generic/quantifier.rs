use build_script_shared::compose_test;
use build_script_shared::error::*;
use build_script_shared::parsers;
use build_script_shared::parsers::*;
use build_script_shared::InputType;
use nom::IResult;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::ContextError;
use nom::error::ParseError;
use nom::error::context;
use nom::sequence::*;
use nom::branch::alt;
use std::fmt::Display;
use std::ops::Bound;
use fake::*;
use rand::Rng;

#[derive(Debug, Clone, Hash, Dummy)]
pub struct Quantifier<I> {
    #[dummy(faker = "EdgeBound")]
    pub max: Bound<u32>,
    marker: Mark<I>,
}

impl<I> Quantifier<I> {
    pub fn new(max: Bound<u32>, marker: Mark<I>) -> Self {
        Quantifier { max, marker }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, f: F) -> Quantifier<O> 
    where
        F: FnMut(I) -> O,
    {
        Quantifier {
            max: self.max,
            marker: self.marker.map(f)
        }
    }
}

/// Matches (=)?<(=)?
fn ls_gt_eq<I, E>(s: I) -> IResult<I, Option<char>, E> 
where
    I: InputType,
    E: ParseError<I> + ContextError<I>
{
    alt((
        preceded(char('<'), opt(char('='))), 
        terminated(opt(char('=')), char('<'))
    ))(s)
}

impl<I: InputType> ParserDeserialize<I> for Quantifier<I> {
    fn parse(s: I) -> ParserResult<I, Self> {
        // Match ls_gt_eq \d
        let greater_than = opt(pair(
            ws(ls_gt_eq), 
            ws(parsers::u32)
        ));
        // match [ (<less_than>)? n (<greater_than>)? ]
        // This does mean [n] is valid. But should not be a problem
        let (s, (res, marker)) = context(
            "Parsing Quantifier",
            marked(opt(surrounded(
                '[',
                tuple((char('n'), greater_than)),
                ']',
            )))
        )(s)?;

        if let Some((_, gt)) = res {
            // Find maximum value
            let max = gt.map(|(eq, count)| if eq.is_some() { Bound::Included(count) } else { Bound::Excluded(count) }).unwrap_or_else(|| Bound::Unbounded );

            Ok((s, Quantifier { max, marker }))
        } else {
            Ok((
                s,
                Quantifier {
                    max: Bound::Unbounded,
                    marker,
                },
            ))
        }
    }
}

impl<I> ParserSerialize for Quantifier<I> {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> ComposerResult<()> {
        match self.max {
            Bound::Excluded(gt) => write!(f, "[n<{}]", gt)?,
            Bound::Included(gt) => write!(f, "[n=<{}]", gt)?,
            Bound::Unbounded => (),
        }
        Ok(())
    }
}

impl<I: Default> Default for Quantifier<I> {
    fn default() -> Self {
        Quantifier::new(Bound::Unbounded, Mark::null())
    }
}

impl<I> Display for Quantifier<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max {
            Bound::Excluded(gt) => write!(f, "[n<{}]", gt)?,
            Bound::Included(gt) => write!(f, "[n=<{}]", gt)?,
            Bound::Unbounded => (),
        }
        Ok(())
    }
}

impl<I> PartialEq for Quantifier<I> {
    fn eq(&self, other: &Self) -> bool {
        self.max.eq(&other.max)
    }
}

impl<I> Eq for Quantifier<I> {}

impl<I> PartialOrd for Quantifier<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I> Ord for Quantifier<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let max = match self.max {
            Bound::Included(i) => Some(i),
            Bound::Excluded(i) => Some(i - 1),
            Bound::Unbounded => None
        };
        let max_other = match other.max {
            Bound::Included(i) => Some(i),
            Bound::Excluded(i) => Some(i - 1),
            Bound::Unbounded => None
        };

        max.cmp(&max_other)
    }
}

impl<I> Marked<I> for Quantifier<I> {
    fn marker(&self) -> &Mark<I> {
        &self.marker
    }
}

#[test]
fn compose_quantitfier() {
    use std::ops::Bound;

    let action = Quantifier {
        max: Bound::Included(10),
        marker: Mark::new("[ n <= 10 ]")
    };

    let s = action.serialize_to_string().unwrap();
    let new_action = Quantifier::parse(s.as_str());
    assert_eq!(new_action, Ok(("", action)));
}

pub(crate) struct EdgeBound;
impl Dummy<EdgeBound> for Bound<u32> {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &EdgeBound, rng: &mut R) -> Self {
        match rng.gen_range(0..=2) {
            0 => Bound::Included(rng.gen()),
            1 => Bound::Excluded(rng.gen()),
            _ => Bound::Unbounded
        }
    }
}

compose_test!{quantity_compose, Quantifier<I>}