use crate::book_generation::build_doc_book;
use crate::book_generation::build_doc_book_with_target;
use crate::book_generation::create_doc_book;
use crate::GenResult;
use crate::Project;
use clap::Parser;
use tempdir::TempDir;
use std::path::Path;
use std::path::PathBuf;

use crate::cli::*;

/// Exports the schemas in the project as an mdbook
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Doc {
    /// Optional path to project folder
    #[clap(flatten)]
    pub settings: ProjectSettings,

    /// Set build directory for docs, defaults to temporary directory
    #[clap(short, long)]
    pub out_dir: Option<PathBuf>,

    /// Where to save the resulting docs, defaults to book folder inside build directory
    #[clap(short, long)]
    pub target_dir: Option<PathBuf>,
}

impl Process<ProjectSettings> for Doc {
    fn process(&self, settings: &ProjectSettings) -> GenResult<()> {

        let target_dir = if let Some(target_dir) = &self.target_dir {
            Some(target_dir.to_path_buf())
        } else if self.out_dir.is_none() {
            Some(Path::new("book").to_path_buf())
        } else {
            None
        };

        let p = self.settings.clone().chain(settings).get_project_path();
        let project = Project::open_project(&p)?;

        // Keep the dir alive until the end of the function
        let mut tmp_dir = None;

        
        let out_dir = if let Some(out_dir) = &self.out_dir {
            println!("Creating docs build dir in {}", out_dir.to_string_lossy());
            create_doc_book(&project, &out_dir)?;
            out_dir.clone()
        } else {
            let tmp = TempDir::new("typed_graph")?;
            let tmp_path = tmp.path().to_path_buf();
            println!("Creating docs build dir in {}", tmp_path.to_string_lossy());
            create_doc_book(&project, &tmp_path)?;
            tmp_dir = Some(tmp);
            tmp_path
        };

        if let Some(target) = target_dir {
            println!("Building docs to {}", target.to_string_lossy());
            build_doc_book_with_target(out_dir, target)?;
        } else {
            println!("Building docs to {}", out_dir.join("book").to_string_lossy());
            build_doc_book(&out_dir)?;
        }

        drop(tmp_dir);

        Ok(())
    }
}
