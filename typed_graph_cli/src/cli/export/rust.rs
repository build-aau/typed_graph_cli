use crate::CodeGenerator;
use crate::GenResult;
use crate::Project;
use clap::Parser;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::cli::*;
use crate::targets;

/// Export the project to a format compatible with typed_graph found on crates.io
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Rust {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Where to output the interface
    #[clap()]
    pub output: PathBuf,
}

impl Process<ProjectSettings> for Rust {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.clone().chain(settings).get_project_path();

        if !self.output.is_dir() {
            create_dir_all(&self.output)?;
        }

        let project = Project::open_project(&p)?;
        CodeGenerator::<targets::Rust>::write_to_file(&project, &self.output)?;

        println!("Done exporting to {:?}", &self.output);

        Ok(())
    }
}
