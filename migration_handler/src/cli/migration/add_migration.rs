use clap::Parser;
use crate::GenResult;
use crate::Project;

use crate::cli::*;

/// Create a new schema from a head and attach a changeset
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AddMigration {
    
    #[clap(flatten)]
    settings: ProjectSettings,

    /// Optional name of schema to add migration to
    /// If none is proved the head will be selected instead
    #[clap()]
    schema: Option<String>,

}

impl Process<ProjectSettings> for AddMigration {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings
            .chain(settings)
            .get_project_path();

        let mut project = Project::open_project(p)?;
        let heads = project.find_heads();

        let mut is_added = false;
        if let Some(head) = &self.schema {
            if heads.contains(head) {
                let new_schema = project.copy_schema(head, true)?;
                let new_changset = project.create_changeset(head, &new_schema)?;
                let p = project.save_changeset(&new_changset)?;
                println!("Saved changeset for {} to {:?}", new_schema, p);
                is_added = true;
            } else {
                println!("{} is not a head", head);
            }
        }

        if !is_added {
            if heads.len() > 1 && self.schema.is_none() {
                println!("Multiple heads present");
            }

            if heads.len() > 1 || self.schema.is_some() {
                println!("Possible heads are");
                for head in heads {
                    println!("  - {:}", head);
                }
            } else if self.schema.is_none() && heads.len() == 1 {
                let head = &heads[0];
                let new_schema = project.copy_schema(head, true)?;
                let new_changset = project.create_changeset(head, &new_schema)?;
                let p = project.save_changeset(&new_changset)?;
                println!("Saved changeset for {} to {:?}", new_schema, p);
            }
        }

        Ok(())
    }
}