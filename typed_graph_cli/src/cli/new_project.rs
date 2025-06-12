use crate::GenResult;
use crate::Project;
use build_script_lang::schema::Schema;
use build_script_shared::parsers::Comments;
use build_script_shared::parsers::Ident;
use build_script_shared::parsers::Mark;
use build_script_shared::InputMarker;
use clap::Parser;

use crate::cli::*;
use std::fs::create_dir_all;

/// Create a new project with a one empty schema
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct NewProject {
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Optional name of the empty schema
    #[clap(short, long)]
    pub name: Option<String>,
}

impl Process<ProjectSettings> for NewProject {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.chain(settings).get_project_path();

        if !p.exists() {
            create_dir_all(&p)?;
        }

        let mut prj = Project::create_project(&p)?;

        // Create an empty schema as a starting point
        if prj.iter_schema().count() == 0 {
            let name = self
                .name
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| "V0.0");
            let input = Mark::new(InputMarker::new_from_file(name.to_string(), p.join("schemas").join(name).join("schema.bs").to_string_lossy().to_string()));
            let id = prj.add_schema(Schema::new(
                Comments::default(),
                Ident::new(name, input.clone()),
                None,
                Vec::default(),
                input
            ))?;
            let schema_path = prj.save_schema(&id)?;
            println!("Created new project in {:?}", schema_path);
        } else {
            println!("Porject already exists");
        }

        Ok(())
    }
}
