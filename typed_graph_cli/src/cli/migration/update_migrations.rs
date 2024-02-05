use crate::GenResult;
use crate::Project;
use clap::Parser;

use crate::cli::*;

/// Update hashes of changesets (default to update all heads)
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct UpdateMigrations {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Update hashes for all changesets in the project
    #[clap(short, long)]
    pub all: bool,
}

impl Process<ProjectSettings> for UpdateMigrations {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.chain(settings).get_project_path();

        let mut project = Project::open_project_raw(p)?;

        if !self.all {
            let heads = project.find_heads();
            let changesets: Vec<_> = project.iter_changesets().copied().collect();
            for changeset_id in changesets {
                let changeset = project.get_changeset(&changeset_id)?;
                if !heads.contains(&changeset.new_version) {
                    continue;
                }

                project.update_changeset(&changeset_id)?;
            }
        } else {
            let changeset_ids: Vec<_> = project.iter_changesets().copied().collect();
            for changeset_id in changeset_ids {
                project.update_changeset(&changeset_id)?;
            }
        }

        Ok(())
    }
}
