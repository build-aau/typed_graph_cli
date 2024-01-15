use crate::cli::*;

use clap::Parser;
use crate::GenResult;

/// Command line interface for managing and auto generating typed_graph interfaces
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(flatten)]
    settings: ProjectSettings,

    #[clap(subcommand)]
    cmd: ArgsType,
}

#[derive(Parser, Debug)]
pub enum ArgsType {
    Migration(Migration),
    Export(Export),
    Schema(Schema),
    List(List),
    New(NewProject)
}

impl Process<()> for Args {
    fn process(&self, _meta: &()) -> GenResult<()> {
        match &self.cmd {
            ArgsType::Migration(migration) => migration.process(&self.settings),
            ArgsType::Schema(schema) => schema.process(&self.settings),
            ArgsType::List(list) => list.process(&self.settings),
            ArgsType::Export(export) => export.process(&self.settings),
            ArgsType::New(new) => new.process(&self.settings)
        }
    }
}
