use std::hash::Hash;

use super::{ws, ComposeContext, ParserDeserialize, ParserSerialize};
use crate::compose_test;
use crate::error::ParserResult;
use crate::input_marker::InputType;
use fake::faker::lorem::en::*;
use fake::{Dummy, Faker};
use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::{context, ContextError, ParseError};
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;
use serde::{Deserialize, Serialize};

/// Store an ordered list of comments
#[derive(Debug, Clone, Default, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Comments {
    comments: Vec<Comment>,
}

/// each comment contains a non zero length string with the text of the comment
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash, Dummy, Serialize, Deserialize)]
#[serde(tag = "type", content = "text")]
pub enum Comment {
    /// Represent a block comment on the form /* abc */
    Block(#[dummy(faker = "Paragraph(1..2)")] String),
    /// Represent a line comment on the form // abc
    Line(#[dummy(faker = "Sentence(2..6)")] String),
    /// Represent a doc comment on the form /// abc
    ///
    /// Doc comment are different from line comments
    /// in that these are often passed along with the data they were written with.
    ///
    /// Line comments are mostly there to allow for comments for the developer not the end user
    Doc(#[dummy(faker = "Sentence(2..6)")] String),
}

impl Comments {
    pub fn new(comments: Vec<Comment>) -> Self {
        Comments { comments }
    }

    pub fn strip_comments(&mut self) {
        self.comments
            .retain(|comment| matches!(comment, Comment::Doc(_)));
    }

    /// Create a new Comments only contianing the doc comments
    pub fn get_doc_comments(&self) -> Comments {
        Comments::new(
            self.comments
                .iter()
                .filter(|comment| matches!(comment, Comment::Doc(_)))
                .cloned()
                .collect(),
        )
    }

    /// Update all doc comments with those from another Comments
    pub fn replace_doc_comments(&mut self, other: &Comments) {
        self.comments = self
            .comments
            .iter()
            .cloned()
            .filter(|comment| !matches!(comment, Comment::Doc(_)))
            .chain(other.get_doc_comments().comments.into_iter())
            .collect();
    }

    /// Iterate through all the doc comments
    pub fn iter_doc(&self) -> impl Iterator<Item = &String> {
        self.comments.iter().filter_map(|comment| {
            if let Comment::Doc(comment) = comment {
                Some(comment)
            } else {
                None
            }
        })
    }

    pub fn iter_non_doc(&self) -> impl Iterator<Item = &String> {
        self.comments.iter().filter_map(|comment| {
            match comment {
                Comment::Block(c) => Some(c),
                Comment::Line(c) => Some(c),
                Comment::Doc(_) => None
            }
        })
    }

    pub fn has_doc(&self) -> bool {
        self.comments.iter().any(|c| matches!(c, Comment::Doc(_)))
    }
}

impl<I: InputType> ParserDeserialize<I> for Comments {
    fn parse(s: I) -> ParserResult<I, Self> {
        let (s, comments) = many0(ws(Comment::parse))(s)?;
        Ok((
            s,
            Comments {
                comments: comments.into_iter().collect(),
            },
        ))
    }
}

impl ParserSerialize for Comments {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
        for comment in &self.comments {
            comment.compose(f, ctx)?;
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
            )),
        )(s)?;

        Ok((s, comment))
    }
}

impl ParserSerialize for Comment {
    fn compose<W: std::fmt::Write>(
        &self,
        f: &mut W,
        ctx: ComposeContext,
    ) -> crate::error::ComposerResult<()> {
        let indent = ctx.create_indents();
        match self {
            Comment::Doc(s) => write!(f, "{indent}///{}", s),
            Comment::Line(s) => write!(f, "{indent}//{}", s),
            Comment::Block(s) => write!(f, "{indent}/*{}*/", s),
        }?;
        Ok(())
    }
}

impl PartialEq for Comments {
    fn eq(&self, other: &Self) -> bool {
        let own_doc_comments: Vec<_> = self.iter_doc().collect();
        let other_doc_comments: Vec<_> = other.iter_doc().collect();

        own_doc_comments == other_doc_comments
    }
}

impl Eq for Comments {}

impl Hash for Comments {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for comment in self.iter_doc() {
            comment.hash(state);
        }
    }
}

impl Comment {
    pub fn doc_line_comment<I, E>(s: I) -> IResult<I, Comment, E>
    where
        I: InputType,
        E: ParseError<I> + ContextError<I>,
    {
        let (s, comment) = context(
            "Parsing DocComment",
            delimited(
                pair(multispace0, tag("///")),
                recognize(many0(preceded(
                    not(char('\n')),
                    // Not does not consume any input so we have to do it manually
                    take(1usize),
                ))),
                alt((value((), char('\n')), value((), eof))),
            ),
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
                pair(multispace0, tag("//")),
                recognize(many0(preceded(
                    not(char('\n')),
                    // Not does not consume any input so we have to do it manually
                    take(1usize),
                ))),
                alt((value((), char('\n')), value((), eof))),
            ),
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
                pair(multispace0, tag("/*")),
                recognize(many0(preceded(
                    not(tag("*/")),
                    // Not does not consume any input so we have to do it manually
                    take(1usize),
                ))),
                context("Expected block comment closing", tag("*/")),
            ),
        )(s)?;

        Ok((s, Comment::Block(comment.to_string())))
    }
}

impl Dummy<Faker> for Comments {
    fn dummy_with_rng<R: rand::prelude::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
        let count = rng.gen_range(0..3);

        Comments {
            comments: (0..count)
                .map(|_| Comment::dummy_with_rng(config, rng))
                .collect(),
        }
    }
}

compose_test! {comment_compose_test, Comment no hash}
compose_test! {comments_compose_test, Comments}
