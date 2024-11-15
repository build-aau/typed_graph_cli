use std::path::PathBuf;

use serde::Serialize;

#[derive(Serialize)]
pub struct StatementDocContext {
    pub title: String,
    pub path: PathBuf
}