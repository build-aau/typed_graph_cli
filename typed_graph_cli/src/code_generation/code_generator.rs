use std::collections::HashMap;
use std::fs::{write, File};
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::GenResult;

pub trait CodeGenerator<Target> {
    fn get_filename(&self) -> String;
    fn aggregate_content<P: AsRef<Path>>(&self, p: P) -> GenResult<GeneratedCode>;

    fn write_to_file<P: AsRef<Path>>(&self, p: P) -> GenResult<()> {
        self.aggregate_content(p)?.write_all()
    }
}

/// A flat container of all the written content to different files
///
/// This reduces the likelyhood of conflicting writes to files as the files are only written to once
#[derive(Debug)]
pub struct GeneratedCode {
    new_files: HashMap<PathBuf, Option<String>>,
}

impl GeneratedCode {
    pub fn new() -> GeneratedCode {
        GeneratedCode {
            new_files: Default::default(),
        }
    }

    pub fn add_content(&mut self, path: PathBuf, content: String) {
        let current_content = self.new_files.entry(path).or_default();
        if let Some(c) = current_content {
            *c += &content;
        } else {
            *current_content = Some(content);
        }
    }

    pub fn create_file(&mut self, path: PathBuf) {
        if !self.new_files.contains_key(&path) {
            self.new_files.insert(path, None);
        }
    }

    pub fn create_file_with_default(&mut self, path: PathBuf, content: String) {
        if !path.exists() {
            self.add_content(path, content);
        }
    }

    pub fn append(&mut self, other: GeneratedCode) {
        for (p, c) in other.new_files {
            if let Some(content) = c {
                self.add_content(p, content);
            } else {
                self.create_file(p);
            }
        }
    }

    pub fn write_all(&self) -> GenResult<()> {
        for (p, c) in &self.new_files {
            if let Some(content) = c {
                if !p.exists() {
                    File::create(&p)?;
                }

                let mut f = File::open(p)?;
                let mut current_content = String::new();
                f.read_to_string(&mut current_content)?;

                if content != &current_content {
                    write(p, content)?;
                }
            } else {
                if !p.exists() {
                    File::create(p)?;
                }
            }
        }
        Ok(())
    }
}

impl Deref for GeneratedCode {
    type Target = HashMap<PathBuf, Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.new_files
    }
}