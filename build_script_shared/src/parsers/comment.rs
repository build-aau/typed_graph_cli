use std::collections::BTreeMap;
use std::hash::Hash;

use crate::compose_test;
use crate::input_marker::InputType;
use crate::error::ParserResult;
use super::{ws, ParserDeserialize, ParserSerialize};
use fake::Dummy;
use nom::IResult;
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::error::{context, ContextError, ParseError};
use nom::multi::*;
use nom::branch::alt;
use nom::sequence::*;
use nom::character::complete::*;
use fake::faker::lorem::en::*;

/// Store an ordered list of comments
#[derive(Debug, Clone, Default, PartialOrd, Ord, Dummy)]
pub struct Comments {
    /// the comments are stored in a BTreeMap to keep the hash consisten
    comments: BTreeMap<usize, Comment>,
}

/// each comment contains a non zero length string with the text of the comment 
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Dummy)]
pub enum Comment {
    /// Represent a block comment on the form /* abc */
    Block(
        #[dummy(faker = "Paragraph(1..2)")]
        String
    ),
    /// Represent a line comment on the form // abc
    Line(
        #[dummy(faker = "Sentence(2..6)")]
        String
    ),
    /// Represent a doc comment on the form /// abc
    /// 
    /// Doc comment are different from line comments
    /// in that these are often passed along with the data they were written with.
    /// 
    /// Line comments are mostly there to allow for comments for the developer not the end user
    Doc(
        #[dummy(faker = "Sentence(2..6)")]
        String
    )
}

impl Comments {
    pub fn new(comments: BTreeMap<usize, Comment>) -> Self {
        Comments {
            comments
        }
    }

    /// Create a new Comments only contianing the doc comments
    pub fn get_doc_comments(&self) -> Comments {
        Comments::new(
            self.comments
                .iter()
                .filter(|(_, comment)| matches!(comment, Comment::Doc(_)))
                .map(|(_, comment)| comment)
                .cloned()
                .enumerate()
                .collect()
        )
    }

    /// Update all doc comments with those from another Comments
    pub fn replace_doc_comments(&mut self, other: &Comments) {
        self.comments = self.comments
                .iter()
                .map(|(_, comment)| comment)
                .cloned()
                .filter(|comment| !matches!(comment, Comment::Doc(_)))
                .chain(other.get_doc_comments().comments.into_iter().map(|(_, comment)| comment))
                .enumerate()
                .collect();
    }

    /// Iterate through all the doc comments
    pub fn iter_doc(&self) -> impl Iterator<Item = &String> {
        self
            .comments
            .iter()
            .map(|(_, b)| b)
            .filter_map(|comment| if let Comment::Doc(s) = comment { 
                Some(s) 
            } else { 
                None 
            })
    }

    pub fn has_doc(&self) -> bool {
        self
            .comments
            .iter()
            .any(|(_, c)| matches!(c, Comment::Doc(_)))
    }
}

impl<I: InputType> ParserDeserialize<I>  for Comments {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = many0(ws(Comment::parse))(s)?;
        Ok((
            s,
            Comments { 
                comments: comments.into_iter().enumerate().collect()
            }
        ))
    }
}

impl ParserSerialize for Comments {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        for (_, comment) in &self.comments {
            comment.compose(f)?;
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<I: InputType> ParserDeserialize<I> for Comment {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comment) = context(
            "Parsing comments",       
                alt((
                    Comment::doc_line_comment,
                    Comment::line_comment,
                    Comment::block_comment,
                ))
        )(s)?;


        Ok((s, comment))
    }
}

impl ParserSerialize for Comment {
    fn compose<W: std::fmt::Write>(&self, f: &mut W) -> crate::error::ComposerResult<()> {
        match self {
            Comment::Doc(s) => write!(f, "///{}", s),
            Comment::Line(s) => write!(f, "//{}", s),
            Comment::Block(s) => write!(f, "/*{}*/", s)
        }?;
        Ok(())
    }
}

impl Hash for Comments {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (_, comment) in &self.comments {
            if let Comment::Doc(s) = comment {
                s.hash(state);
                s.hash(state)
            }
        }
    }
}

impl PartialEq for Comments {
    fn eq(&self, other: &Self) -> bool {
        if self.comments.len() != other.comments.len() {
            return false;
        }

        let iter = self.comments.iter().zip(other.comments.iter());
        for ((_, comment), (_, other_comment)) in iter {
            if comment != other_comment {
                return false;
            }
        }

        true
    }
}

impl Eq for Comments {}

impl Comment {
    
    pub fn doc_line_comment<I, E>(s: I) -> IResult<I, Comment, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, comment) = context(
            "Parsing DocComment",
            delimited(
                pair(
                    multispace0,
                    tag("///")
                ), 
                recognize(many0(preceded(
                    not(char('\n')), 
                    // Not does not consume any input so we have to do it manually
                    take(1usize)
                ))),
                alt((
                    value( (), char('\n')),
                    value( (), eof)
                ))
            )
        )(s)?;
    
        Ok((s, Comment::Doc(comment.to_string())))
    }

    pub fn line_comment<I, E>(s: I) -> IResult<I, Comment, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, comment) = context(
            "Parsing LineComment",
            delimited(
                pair(
                    multispace0,
                    tag("//")
                ), 
                recognize(many0(preceded(
                    not(char('\n')), 
                    // Not does not consume any input so we have to do it manually
                    take(1usize)
                ))),
                alt((
                    value( (), char('\n')),
                    value( (), eof)
                ))
            )
        )(s)?;
    
        Ok((s, Comment::Line(comment.to_string())))
    }
    
    pub fn block_comment<I, E>(s: I) -> IResult<I, Comment, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, comment) = context(
            "Parsing BlockComment",
            delimited(
                pair(
                    multispace0,
                    tag("/*")
                ),
                recognize(many0(preceded(
                    not(tag("*/")), 
                    // Not does not consume any input so we have to do it manually
                    take(1usize)
                ))),
                context("Expected block comment closing", tag("*/"))
            )
        )(s)?;
    
        Ok((s, Comment::Block(comment.to_string())))
    }
}

compose_test!{comment_compose_test, Comment no hash}
compose_test!{comments_compose_test, Comments}