use crate::cli::*;
use crate::{Direction, GenResult, Project};
use clap::Parser;
use std::collections::HashMap;

/// List all schemas and changesets in the current project
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct List {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,
}

impl Process<ProjectSettings> for List {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {
        let p = self.settings.chain(settings).get_project_path();

        let project = Project::open_project(p)?;

        let version_map: HashMap<&String, Vec<&String>> = project
            .iter_version(Some(Direction::Backwards))
            .fold(HashMap::new(), |mut acc, (old, new, _)| {
                let parents = acc.entry(old).or_default();
                parents.push(new);

                acc
            });

        let heads = project.find_heads();
        let mut to_visit: Vec<_> = heads.iter().map(|s| (s, 0)).collect();

        while let Some((schema, indents)) = to_visit.pop() {
            if indents > 0 {
                for _ in 0..(indents - 1) {
                    print!("│  ");
                }
                print!("├──");
            }
            println!("{schema}");

            let children = version_map
                .get(schema)
                .into_iter()
                .flat_map(|v| v.into_iter());

            for child in children {
                to_visit.push((*child, indents + 1));
            }
        }

        Ok(())
    }
}
