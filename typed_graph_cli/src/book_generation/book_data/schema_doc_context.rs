use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::GenResult;

use super::StatementDocContext;

#[derive(Serialize)]
pub struct SchemaDocContext {
    title: String,
    path: String,

    pub schema_path: PathBuf,

    nodes: Vec<StatementDocContext>,
    edges: Vec<StatementDocContext>,
    types: Vec<StatementDocContext>,
    structs: Vec<StatementDocContext>,
    imports: Vec<StatementDocContext>,
}

impl SchemaDocContext {
    pub fn new(title: String, book_src_path: PathBuf) -> GenResult<Self> {
        let schema_path = book_src_path.join(&title);
        let path = format!("{title}.md");

        let node_path = schema_path.join("nodes");
        let edge_path = schema_path.join("edges");
        let types_path = schema_path.join("types");
        let structs_path = schema_path.join("structs");
        let imports_path = schema_path.join("imports");

        create_dir_all(&node_path)?;
        create_dir_all(&edge_path)?;
        create_dir_all(&types_path)?;
        create_dir_all(&structs_path)?;
        create_dir_all(&imports_path)?;

        Ok(SchemaDocContext {
            title,

            path,

            schema_path,

            nodes: Default::default(),
            edges: Default::default(),
            types: Default::default(),
            structs: Default::default(),
            imports: Default::default(),
        })
    }

    pub fn add_node_section(&mut self, title: String, content: String) -> GenResult<()>{
        SchemaDocContext::add_section(title, content, &self.schema_path, Path::new("nodes"), &mut self.nodes)
    }

    pub fn add_edge_section(&mut self, title: String, content: String) -> GenResult<()>{
        SchemaDocContext::add_section(title, content, &self.schema_path, Path::new("edges"), &mut self.edges)
    }

    pub fn add_struct_section(&mut self, title: String, content: String) -> GenResult<()>{
        SchemaDocContext::add_section(title, content, &self.schema_path, Path::new("structs"), &mut self.structs)
    }

    pub fn add_type_section(&mut self, title: String, content: String) -> GenResult<()>{
        SchemaDocContext::add_section(title, content, &self.schema_path, Path::new("types"), &mut self.types)
    }

    fn add_section(title: String, content: String, schema_path: &Path, relative_path: &Path, stm_list: &mut Vec<StatementDocContext>) -> GenResult<()>{
        let relative_path = relative_path.join(format!("{title}.md"));
        let mut f = File::create(&schema_path.join(&relative_path))?;
        write!(f, "{}", content)?;

        stm_list.push(StatementDocContext {
            title,
            path: relative_path
        });

        Ok(())
    }
}