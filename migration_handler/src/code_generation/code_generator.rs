use std::collections::HashMap;
use std::fs::{write, File};
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
    new_files: HashMap<PathBuf, String>
}

impl GeneratedCode {
    pub fn new() -> GeneratedCode {
        GeneratedCode { 
            new_files: Default::default() 
        }
    }

    pub fn add_content(&mut self, path: PathBuf, content: String) {
        let current_content = self.new_files.entry(path).or_default();
        *current_content += &content;
    }

    pub fn create_file(&mut self, path: PathBuf) {
        self.add_content(path, "".to_string())
    }

    pub fn append(&mut self, other: GeneratedCode) {
        for (p, c) in other.new_files {
            self.add_content(p, c);
        }
    }

    pub fn write_all(&self) -> GenResult<()> {
        for (p, c) in &self.new_files {
            if c.is_empty() {
                if !p.exists() {
                    File::create(p)?;
                }
            } else {
                write(p, c)?;
            }
        }
        Ok(())
    }
}