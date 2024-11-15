use std::fs::{create_dir_all, File};
use std::io::{Write};
use std::path::{Path, PathBuf};

use tera::Context;

use crate::book_generation::copy_resources;
use crate::GenResult;

use super::SchemaDocContext;

pub struct DocBookContext {
    pub book_root_path: PathBuf,
    pub book_src_path: PathBuf,
    pub book_res_path: PathBuf,
    main_sections: Vec<SchemaDocContext>,
    other_sections: Vec<SchemaDocContext>,
}

impl DocBookContext {
    pub fn new(book_root_path: impl AsRef<Path>) -> GenResult<Self> {
        let out_dir = book_root_path.as_ref();
        if !out_dir.exists() {
            create_dir_all(out_dir)?;
        }
        let out_src_dir = out_dir.join("src");
        if !out_src_dir.exists() {
            create_dir_all(&out_src_dir)?;
        }

        let out_res_dir = out_src_dir.join("resources");
        if !out_res_dir.exists() {
            create_dir_all(&out_res_dir)?;
        }

        copy_resources(out_dir)?;
        
        Ok(DocBookContext {
            book_root_path: out_dir.to_path_buf(),
            book_src_path: out_src_dir,
            book_res_path: out_res_dir,
            other_sections: Default::default(),
            main_sections: Default::default()
        })
    }

    pub fn create_main_section(&mut self, title: String, content: String) -> GenResult<&mut SchemaDocContext> {
        let content_path = self.book_src_path.join(format!("{title}.md"));
        let mut f = File::create(&content_path)?;
        writeln!(f, "# {title}")?;
        write!(f, "{}", content)?;

        let section = SchemaDocContext::new(title, self.book_src_path.clone())?;
        self.main_sections.push(section);
        Ok(self.main_sections.last_mut().unwrap())
    }

    pub fn create_other_section(&mut self, title: String, content: String) -> GenResult<&mut SchemaDocContext> {
        let content_path = self.book_src_path.join(format!("{title}.md"));
        let mut f = File::create(&content_path)?;
        writeln!(f, "# {title}")?;
        write!(f, "{}", content)?;

        let section = SchemaDocContext::new(title, self.book_src_path.clone())?;
        self.other_sections.push(section);
        Ok(self.other_sections.last_mut().unwrap())
    }

    pub fn add_to_context(&self, ctx: &mut Context) {
        ctx.insert("main_sections", &self.main_sections);
        ctx.insert("other_sections", &self.other_sections);
    }
}