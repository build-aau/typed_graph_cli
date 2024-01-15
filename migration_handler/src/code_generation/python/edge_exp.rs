use build_script_lang::schema::{EdgeExp, Schema};
use std::fmt::Write;
use std::path::Path;

use crate::{CodeGenerator, GeneratedCode, GenResult, ToSnakeCase, targets, ToPythonType};

impl CodeGenerator<targets::Python> for EdgeExp<String> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(&self, p: P) -> crate::GenResult<GeneratedCode> {
        let edge_name = &self.name;
        let edges_path = p.as_ref().join(format!("{}.py", CodeGenerator::<targets::Python>::get_filename(self)));
        let mut s = String::new();

        writeln!(s, "from ..edge_type import EdgeType")?;
        writeln!(s, "from ..edge import Edge")?;
        writeln!(s, "from ..structs import *")?;
        writeln!(s, "from ..types import *")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s, "from typing import Optional, List, Dict")?;
        writeln!(s, "")?;
        writeln!(s, "class {edge_name}(Edge):")?;
        if self.comments.has_doc() {
            writeln!(s, "     \"\"\"")?;
        for comment in self.comments.iter_doc() {
                writeln!(s, "     {comment}")?;
            }
            writeln!(s, "     \"\"\"")?;
        }
        writeln!(s, "     id: EdgeId")?;
        for (name, field_value) in &self.fields.fields {
            let field_type = field_value.ty.to_python_type();
            writeln!(s, "     {name}: {field_type}")?;
            if field_value.comments.has_doc() {
                writeln!(s, "     \"\"\"")?;
                for comment in field_value.comments.iter_doc() {
                    writeln!(s, "     {comment}")?;
                }
                writeln!(s, "     \"\"\"")?;
            }
        }
        writeln!(s, "")?;
        writeln!(s, "     def get_type(self) -> EdgeType:")?;
        writeln!(s, "          return EdgeType.{edge_name}")?;
        
        let mut new_files = GeneratedCode::new();
        new_files.add_content(edges_path, s);

        Ok(new_files)
    }
}

// Write ./edges.rs
pub(super) fn write_edges_py(
    new_files: &mut GeneratedCode, 
    schema_folder: &Path
) -> GenResult<()> {
    let edge_path = schema_folder.join("edge.py");

    let mut edge = String::new();

    writeln!(edge, "from typed_graph import EdgeExt")?;
    writeln!(edge, "from .. import imports")?;
    writeln!(edge, "from .edge_type import EdgeType")?;
    writeln!(edge, "")?;
    writeln!(edge, "class Edge(EdgeExt[imports.EdgeId, EdgeType]):")?;
    writeln!(edge, "    def get_id(self) -> imports.EdgeId:")?;
    writeln!(edge, "        return self.id")?;
    writeln!(edge, "    ")?;
    writeln!(edge, "    def set_id(self, id: imports.EdgeId) -> None:")?;
    writeln!(edge, "        self.id = id")?;

    new_files.add_content(edge_path.clone(), edge);

    Ok(())
}

/// Write ./edge_type.rs
pub(super) fn write_edge_type_py(schema: &Schema<String>, new_files: &mut GeneratedCode, schema_folder: &Path) -> GenResult<()> {
    let edge_path = schema_folder.join("edge_type.py");
    let edges: Vec<_> = schema.edges().collect();
    let mut edge_type = String::new();

    writeln!(edge_type, "from typed_graph import StrEnum")?;
    writeln!(edge_type, "")?;
    writeln!(edge_type, "class EdgeType(StrEnum):")?;
    if !edges.is_empty() {
        for n in &edges {
            let name = n.name.to_string();
            writeln!(edge_type, "    {name} = '{name}'")?;
        }
    } else {
        writeln!(edge_type, "    pass")?;
    }

    new_files.add_content(edge_path, edge_type);

    Ok(())
}