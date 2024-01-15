use clap::{Parser, Subcommand};
use crate::{cli::*, GenResult};

#[derive(Parser, Debug)]
pub struct Export {
    #[clap(flatten)]
    settings: ProjectSettings,

    #[clap(subcommand)]
    cmd: ExportType,
    
}

/// Export the a project to different languages
#[derive(Subcommand, Debug)]
pub enum ExportType {
    Rust(Rust),
    Python(Python),
}

impl Process<ProjectSettings> for Export {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {

        match &self.cmd {
            ExportType::Rust(r) => r.process(&self.settings.chain(settings)),
            ExportType::Python(py) => py.process(&self.settings.chain(settings))
        }
    }
}