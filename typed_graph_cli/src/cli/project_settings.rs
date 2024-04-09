use clap::Parser;
use std::path::PathBuf;

/// Project settings
#[derive(Parser, Debug, Default, Clone)]
pub struct ProjectSettings {
    /// Optional path to project folder (defaults to current folder)
    #[clap(short, long)]
    pub project: Option<PathBuf>,
}

impl ProjectSettings {
    pub fn chain(&self, other: &ProjectSettings) -> Self {
        ProjectSettings {
            project: self.project.clone().or_else(|| other.project.clone()),
        }
    }

    pub fn get_project_path(&self) -> PathBuf {
        self.project
            .clone()
            .unwrap_or_else(|| PathBuf::from("project"))
    }
}
