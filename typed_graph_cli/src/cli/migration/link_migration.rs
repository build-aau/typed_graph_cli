use crate::GenResult;
use crate::Project;
use clap::Parser;

use crate::cli::*;

/// Create a changeset between two existing schemas
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct LinkMigration {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    #[clap()]
    pub source: String,

    #[clap()]
    pub target: String,
}

impl Process<ProjectSettings> for LinkMigration {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.chain(settings).get_project_path();

        let mut project = Project::open_project(p)?;
        project.create_changeset(&self.source, &self.target)?;

        Ok(())
    }
}
