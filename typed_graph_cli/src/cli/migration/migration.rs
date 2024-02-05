use crate::{cli::*, GenResult};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Migration {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    // List all schemas and their migrations
    #[clap(subcommand)]
    pub cmd: MigrationType,
}

/// Modify changesets
#[derive(Subcommand, Debug)]
pub enum MigrationType {
    Add(AddMigration),
    Link(LinkMigration),
    Update(UpdateMigrations),
}

impl Process<ProjectSettings> for Migration {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        match &self.cmd {
            MigrationType::Add(a) => a.process(&self.settings.chain(settings)),
            MigrationType::Link(a) => a.process(&self.settings.chain(settings)),
            MigrationType::Update(a) => a.process(&self.settings.chain(settings)),
        }
    }
}
