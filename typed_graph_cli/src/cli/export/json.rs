use crate::GenResult;
use crate::Project;
use clap::Parser;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::cli::*;
use std::fs::File;

/// Exports the schemas in the project json files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Json {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Where to save the json file
    #[clap()]
    pub output: PathBuf,
}

impl Process<ProjectSettings> for Json {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.clone().chain(settings).get_project_path();

        if !self.output.is_dir() {
            create_dir_all(&self.output)?;
        }

        let project = Project::open_project(&p)?;
        for schema_id in project.iter_schema() {
            let schema = project.get_schema(schema_id)?;
            let schema_path = self.output.join(format!("{}.json", schema.version));
            let mut f = File::create(schema_path)?;
            serde_json::to_writer_pretty(&mut f, schema)?;
        }

        println!("Done exporting to {:?}", &self.output);
        Ok(())
    }
}
