use crate::code_preview::CodePreview;
use crate::input_marker::InputMarker;
use nom::InputLength;
use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;

use super::{ParserError, ParserErrorKind};

#[derive(Error, Debug)]
pub struct OwnedParserError {
    pub data: HashMap<String, String>,
    pub errors: Vec<OwnedError>,
}

#[derive(Debug)]
pub struct OwnedError {
    offset: usize,
    len: usize,
    source: String,
    kind: ParserErrorKind,
}

impl<I> From<ParserError<InputMarker<I>>> for OwnedParserError
where
    I: ToString,
{
    fn from(e: ParserError<InputMarker<I>>) -> Self {
        // Find the data of the most complete source file
        let mut data: HashMap<String, String> = HashMap::new();
        for (marker, _) in &e.errors {
            let source_file = marker.source_file.clone();
            let source_data = marker.leak_source().to_string();
            if let Some(current_data) = data.get_mut(&source_file) {
                if source_data.len() > current_data.len() {
                    *current_data = source_data;
                }
            } else {
                data.insert(source_file, source_data);
            }
        }

        OwnedParserError {
            // Find the error with the longest version of the original data
            data,
            errors: e
                .errors
                .iter()
                .map(|(input, kind)| OwnedError {
                    offset: input.source_offset(),
                    len: input.input_len(),
                    source: input.get_source().to_string(),
                    kind: kind.clone(),
                })
                .collect(),
        }
    }
}

impl Display for OwnedParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const ERROR_COUNT: usize = 3;

        // Nom errors are omitted as they do not provide much information
        let displayed_errors = self
            .errors
            .iter()
            .filter(|e| {
                !matches!(
                    e.kind,
                    ParserErrorKind::ErrorKind(_) | ParserErrorKind::EndOfFile
                )
            })
            .take(ERROR_COUNT);

        for e in displayed_errors {
            let source = if e.source.is_empty() {
                "String"
            } else {
                e.source.as_str()
            };

            let preview = if let Some(data) = self.data.get(&e.source) {
                let caret_len = if e.offset + e.len >= self.data.len() {
                    1
                } else {
                    e.len
                };

                let preview = CodePreview::new(data, e.offset, caret_len, 2, 2, true);
                write!(
                    f,
                    "{}:{}:{}: ",
                    source,
                    preview.caret_line_number() + 1,
                    preview.caret_offset() + 1
                )?;
                Some(preview)
            } else {
                write!(f, "{}:1:1: ", source)?;
                None
            };

            writeln!(f, "{}", &e.kind)?;

            if let Some(preview) = preview {
                writeln!(f, "{preview}")?;
            }
        }

        Ok(())
    }
}
