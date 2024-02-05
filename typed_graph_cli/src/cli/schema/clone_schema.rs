use crate::GenResult;
use crate::Project;
use clap::Parser;

use crate::cli::*;

/// Create a clone of an existing schema
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CloneSchema {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Name of the schema to clone
    #[clap()]
    pub schema: String,
}

impl Process<ProjectSettings> for CloneSchema {
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
        let new_schema = project.copy_schema(&self.schema, false)?;
        let schema_path = project.save_schema(&new_schema)?;
        println!("Cloned schema to {:?}", schema_path);

        Ok(())
    }
}
