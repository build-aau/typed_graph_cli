use crate::{cli::*, GenResult};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Schema {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    // List all schemas and their migrations
    #[clap(subcommand)]
    pub cmd: SchemaType,
}

/// Modify schemas
#[derive(Subcommand, Debug)]
pub enum SchemaType {
    Clone(CloneSchema),
    Rename(RenameSchema),
}

impl Process<ProjectSettings> for Schema {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        match &self.cmd {
            SchemaType::Clone(a) => a.process(&self.settings.chain(settings)),
            SchemaType::Rename(a) => a.process(&self.settings.chain(settings)),
        }
    }
}
