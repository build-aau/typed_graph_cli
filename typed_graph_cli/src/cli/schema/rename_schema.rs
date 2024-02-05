use crate::GenResult;
use crate::Project;
use clap::Parser;

use crate::cli::*;

/// Rename a schema and update all
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct RenameSchema {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Name of the schema to rename
    #[clap()]
    pub schema: String,

    /// New name of the schema
    #[clap()]
    pub new_name: String,
}

impl Process<ProjectSettings> for RenameSchema {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.chain(settings).get_project_path();

        let mut project = Project::open_project(p)?;
        if !project.has_schema(&self.schema) {
            println!("Failed to find schema {}", &self.schema);
            println!("Possible schemas are:");
            for schema in project.iter_schema() {
                println!(" - {}", schema);
            }
            return Ok(());
        }
        if project.has_schema(&self.new_name) {
            println!("A schema called {} already exists", self.schema);
            return Ok(());
        }

        project.rename_schema(&self.schema, self.new_name.clone())?;

        Ok(())
    }
}
