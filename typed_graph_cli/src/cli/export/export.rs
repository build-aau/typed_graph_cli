use crate::{cli::*, GenResult};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Export {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    #[clap(subcommand)]
    pub cmd: ExportType,
}

/// Export the a project to different languages
#[derive(Subcommand, Debug)]
pub enum ExportType {
    Rust(Rust),
    Python(Python),
    Json(Json)
}

impl Process<ProjectSettings> for Export {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        match &self.cmd {
            ExportType::Rust(r) => r.process(&self.settings.chain(settings)),
            ExportType::Python(py) => py.process(&self.settings.chain(settings)),
            ExportType::Json(json) => json.process(&self.settings.chain(settings)),
        }
    }
}
