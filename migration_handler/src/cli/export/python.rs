use std::path::PathBuf;
use std::fs::create_dir_all;
use clap::Parser;
use crate::GenResult;
use crate::Project;
use crate::CodeGenerator;

use crate::cli::*;
use crate::targets;

/// Exports the project to a format compatible with typed_graph found on pip
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Python {
    
    /// Optional path to project folder
    #[clap(flatten)]
    settings: ProjectSettings,

    /// Where to output the interface
    #[clap()]
    output: PathBuf,

}

impl Process<ProjectSettings> for Python {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings
            .clone()
            .chain(settings)
            .get_project_path();

        if !self.output.is_dir() {
            create_dir_all(&self.output)?;
        }


        let project = Project::open_project(p)?;
        CodeGenerator::<targets::Python>::write_to_file(&project, &self.output)?;

        Ok(())
    }
}