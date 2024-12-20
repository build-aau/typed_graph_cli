use super::Marked;
use fake::*;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Stores the position of an item in the given input data
#[derive(Clone, Copy, Default, Dummy)]
pub struct Mark<I> {
    marker: I,
}

impl<I> Mark<I> {
    pub fn new(s: I) -> Mark<I> {
        Mark { marker: s }
    }

    /// Move from one input type to another
    pub fn map<O, F>(self, mut f: F) -> Mark<O>
    where
        F: FnMut(I) -> O,
    {
        Mark {
            marker: f(self.marker),
        }
    }
}

impl<I: Default> Mark<I> {
    /// Create an empty marker indicating the origin is unknown
    pub fn null() -> Mark<I> {
        Mark {
            marker: Default::default(),
        }
    }
}

impl<I> Deref for Mark<I> {
    type Target = I;
    fn deref(&self) -> &I {
        &self.marker
    }
}

impl<I> Marked<I> for Mark<I> {
    fn marker(&self) -> &Self {
        self
    }
}

impl<I> Marked<I> for &Mark<I> {
    fn marker(&self) -> &Mark<I> {
        self
    }
}

// Since the mark sometimes include the entire source file we leave it out of the debug.
// That way debug info becomes less cluttered
impl<I> Debug for Mark<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mark {{ /* Mark Reference */}} ")
    }
}

/// The marker is not included in the hash since multiple equal items
/// can originate from different places in the input file
impl<I> Hash for Mark<I> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

/// All markers are equal as long as the originates from the same input type
impl<I> PartialOrd for Mark<I> {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

/// All markers are equal as long as the originates from the same input type
impl<I> Ord for Mark<I> {
    fn cmp(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

/// All markers are equal as long as the originates from the same input type
impl<I> PartialEq for Mark<I> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<I> Eq for Mark<I> {}
