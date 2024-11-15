use std::path::{PathBuf, StripPrefixError};

use build_changeset_lang::ChangeSetError;
use build_script_shared::BUILDScriptError;
use thiserror::Error;

pub type GenResult<T> = Result<T, GenError>;

#[derive(Error, Debug)]
pub enum GenError {
    #[error(transparent)]
    ParserError(#[from] BUILDScriptError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FormatError(#[from] std::fmt::Error),
    #[error("Failed to find project at {0:?} does not exist")]
    InvalidProjectPath(PathBuf),
    #[error("Failed to find schema at {0:?}")]
    InvalidSchemaPath(PathBuf),
    #[error(
        "Attempted to add multiple {kind} for {old}({old_hash:#16x}) -> {new}({new_hash:#16x})"
    )]
    DuplicateKeys {
        kind: String,
        old: String,
        new: String,
        old_hash: u64,
        new_hash: u64,
    },
    #[error("Imported schema with no changesets {name}({id:#16x})")]
    UnusedSchema { name: String, id: u64 },
    #[error("No upgrade path could be found to {target}")]
    UnreachableSchema { target: String },
    #[error("Found no schema called {name}")]
    UnknownSchema { name: String },
    #[error("Found no changeset called {name:#16x}")]
    UnknownChangeset { name: u64 },
    #[error("Found no refence called {name}")]
    UnknownReference { name: String },
    #[error("Failed to find {kind} {missing_key} from version tree")]
    MalformedVersionTree { kind: String, missing_key: String },
    #[error("Expected folder at {folder}")]
    MissingFolder { folder: String },
    #[error("Recieved malformed path")]
    MalformedPath,
    #[error("Export failed due to {0}")]
    ExportFailed(String),
    #[error("Changeset {old_version} -> {new_version} has different hashes for {schema} expected {expected:#16x} recieved {recieved:#16x}")]
    DivergentChangeset {
        old_version: String,
        new_version: String,
        schema: String,
        expected: u64,
        recieved: u64,
    },
    #[error(transparent)]
    ChangeSetError(#[from] ChangeSetError),
    #[error(transparent)]
    PrefixError(#[from] StripPrefixError),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    BookError(#[from] mdbook::errors::Error),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
    #[error(transparent)]
    ClapError(#[from] clap::Error),
}
