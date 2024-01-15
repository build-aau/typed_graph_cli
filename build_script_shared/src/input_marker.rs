use nom::*;
use std::fmt::{Display, Debug};
use std::ops::{RangeFrom, RangeTo, Range};
use std::hash::Hash;

use crate::code_preview::CodePreview;

pub trait InputType:
    Slice<RangeFrom<usize>>
    + Slice<RangeTo<usize>>
    + Slice<Range<usize>>
    + InputIter<Item=char>
    + InputTakeAtPosition<Item=char>
    + Clone
    + for<'a> FindSubstring<&'a str>
    + InputTake
    + for<'a> Compare<&'a str>
    + AsRef<str>
    + ToString
    + Offset
    + InputLength
    + Default
    + Debug
    + PartialEq
    + Hash
{
    type Item: AsChar + Clone + Copy;
}

impl<'a> InputType for &'a str {
    type Item = char;
}

pub type InputMarkerRef<'a> = InputMarker<&'a  str>;
pub type OwnedMarker = InputMarker<String>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct InputMarker<I> {
    data: I,
    pub(crate) source_file: String,
    start: usize,
    end: usize,
}

impl<I> InputMarker<I> {
    pub fn new(s: I) -> InputMarker<I> 
    where
        I: AsRef<str>
    {
        let end = s.as_ref().len();

        InputMarker {
            data: s,
            source_file: "".to_string(),
            start: 0,
            end
        }
    }

    pub fn new_from_file(s: I, source_file: String) -> InputMarker<I> 
    where
        I: AsRef<str>
    {
        let end = s.as_ref().len();

        InputMarker {
            data: s,
            source_file: source_file,
            start: 0,
            end
        }
    }

    pub fn get_source(&self) -> &str {
        &self.source_file
    }

    pub fn source_offset(&self) -> usize {
        self.start
    }

    pub fn leak_source(&self) -> &I {
        &self.data
    }

    pub fn get_preview(&self, lines_above: usize, lines_below: usize) -> CodePreview 
    where
        I: AsRef<str>
    {
        CodePreview::new(
            self.data.as_ref(),
            self.start,
            self.input_len(),
            lines_above,
            lines_below,
        )
    }
}

impl<I> InputType for InputMarker<I> 
where
    I: InputType,
    for<'b> InputMarker<I>: Compare<&'b str>,
    for<'b> InputMarker<I>: FindSubstring<&'b str>
{
    type Item = <I as InputType>::Item;
}

impl<I> AsRef<str> for InputMarker<I> 
where
    I: AsRef<str>
{
    fn as_ref(&self) -> &str {
        self.data.as_ref()[self.start..self.end].as_ref()
    }
}

impl<I> Compare<&str> for InputMarker<I> 
where
    I: AsRef<str>
{
    fn compare(&self, t: &str) -> CompareResult {
        self.as_ref().compare(t)
    }

    fn compare_no_case(&self, t: &str) -> CompareResult {
        self.as_ref().compare_no_case(t)
    }
}

impl<I> Slice<RangeFrom<usize>> for InputMarker<I> 
where
    I: Clone
{
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        InputMarker {
            start: self.end.min(range.start + self.start),
            source_file: self.source_file.clone(),
            end: self.end,
            data: self.data.clone(),
        }
    }
}

impl<I> Slice<RangeTo<usize>> for InputMarker<I> 
where
    I: Clone
{
    fn slice(&self, range: RangeTo<usize>) -> Self {
        InputMarker {
            end: self.start.max(self.start + range.end).min(self.end),
            source_file: self.source_file.clone(),
            start: self.start,
            data: self.data.clone()
        }
    }
}

impl<I> Slice<Range<usize>> for InputMarker<I> 
where
    I: Clone
{
    fn slice(&self, range: Range<usize>) -> Self {
        InputMarker {
            start: self.end.min(range.start + self.start),
            source_file: self.source_file.clone(),
            end: self.start.max(self.start + range.end).min(self.end),
            data: self.data.clone(),
        }
    }
}

impl<I: AsRef<str> + PartialEq> PartialOrd for InputMarker<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<I: AsRef<str> + PartialEq + Eq> Ord for InputMarker<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(&other.as_ref())
    }
}

impl<I> InputLength for InputMarker<I> {
    fn input_len(&self) -> usize {
        self.end - self.start
    }
}

impl<I> InputIter for InputMarker<I> 
where
    I: InputIter<Item = char> + Slice<Range<usize>>
{
    type Item = char;
    type Iter = I::Iter;
    type IterElem = I::IterElem;
    fn iter_indices(&self) -> Self::Iter {
        (&self.data.slice(self.start..self.end)).iter_indices()
    }
    fn iter_elements(&self) -> Self::IterElem {
        (&self.data.slice(self.start..self.end)).iter_elements()
    }
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        (&self.data.slice(self.start..self.end)).position(predicate)
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        (&self.data.slice(self.start..self.end)).slice_index(count)
    }
}

impl<I> UnspecializedInput for InputMarker<I> {}

impl<'a, I> FindSubstring<&'a str> for InputMarker<I> 
where
    I: Slice<Range<usize>> + FindSubstring<&'a str>
{
    fn find_substring(&self, substr: &'a str) -> Option<usize> {
        (&self.data.slice(self.start..self.end)).find_substring(substr)
    }
}

impl<I> InputTake for InputMarker<I> 
where
    I: Clone
{
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<I> Offset for InputMarker<I> {
    fn offset(&self, second: &Self) -> usize {
        second.start - self.start
    }
}

impl<I> Display for InputMarker<I> 
where
    I: AsRef<str>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref().to_string())
    }
}


impl<'a> From<InputMarker<&'a String>> for InputMarker<&'a str> {
    fn from(value: InputMarker<&'a String>) -> Self {
        InputMarker {
            data: value.data.as_str(),
            source_file: value.source_file,
            start: value.start,
            end: value.end
        }
    }
}

impl<'a> From<&'a String> for InputMarker<&'a str> {
    fn from(value: &'a String) -> Self {
        InputMarker::new(value.as_str())
    }
}

impl<'a> From<&'a str> for InputMarker<&'a str> {
    fn from(value: &'a str) -> Self {
        InputMarker::new(value)
    }
}

impl From<String> for InputMarker<String> {
    fn from(value: String) -> Self {
        InputMarker::new(value)
    }
}