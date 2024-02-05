use build_script_lang::schema::{NodeExp, Schema};
use std::fmt::Write;
use std::path::Path;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToPythonType, ToSnakeCase};

impl<I> CodeGenerator<targets::Python> for NodeExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let node_name = &self.name;
        let node_path = p.as_ref().join(format!(
            "{}.py",
            CodeGenerator::<targets::Python>::get_filename(self)
        ));

        let mut s = String::new();

        writeln!(s, "from ..node import Node")?;
        writeln!(s, "from ..node_type import NodeType")?;
        writeln!(s, "from ..structs import *")?;
        writeln!(s, "from ..types import *")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s, "from .. import *")?;
        writeln!(s, "from typing import Optional, List, Dict, Iterable, Tuple")?;
        writeln!(s, "")?;
        writeln!(s, "class {node_name}(Node):")?;
        if self.comments.has_doc() {
            writeln!(s, "    \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "    {comment}")?;
            }
            writeln!(s, "    \"\"\"")?;
        }
        writeln!(s, "    id: NodeId")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            let field_type = field_value.field_type.to_python_type();
            writeln!(s, "    {field_name}: {field_type}")?;
            if field_value.comments.has_doc() {
                writeln!(s, "    \"\"\"")?;
                for comment in field_value.comments.iter_doc() {
                    writeln!(s, "    {comment}")?;
                }
                writeln!(s, "    \"\"\"")?;
            }
        }
        writeln!(s, "")?;
        writeln!(s, "    def get_type(self) -> NodeType:")?;
        writeln!(s, "        return NodeType.{node_name}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}

/// Write ./nodes.rs
pub(super) fn write_nodes_py(new_files: &mut GeneratedCode, schema_folder: &Path) -> GenResult<()> {
    let node_path = schema_folder.join("node.py");

    let mut node = String::new();

    writeln!(node, "from typing import TypeVar")?;
    writeln!(node, "from typed_graph import NodeExt")?;
    writeln!(node, "from .node_type import NodeType")?;
    writeln!(node, "from .. import imports")?;
    writeln!(node, "")?;
    writeln!(node, "class Node(NodeExt[imports.NodeId, NodeType]):")?;
    writeln!(node, "    def get_id(self) -> imports.NodeId:")?;
    writeln!(node, "        return self.id")?;
    writeln!(node, "    ")?;
    writeln!(node, "    def set_id(self, id: imports.NodeId) -> None:")?;
    writeln!(node, "        self.id = id")?;

    new_files.add_content(node_path, node);

    Ok(())
}

/// Write ./node_type.rs
pub(super) fn write_node_type_py<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let node_path = schema_folder.join("node_type.py");
    let nodes: Vec<_> = schema.nodes().collect();
    let mut node_type = String::new();

    writeln!(node_type, "from typed_graph import StrEnum")?;
    writeln!(node_type, "")?;
    writeln!(node_type, "class NodeType(StrEnum):")?;
    if !nodes.is_empty() {
        for n in &nodes {
            let name = n.name.to_string();
            writeln!(node_type, "    {name} = '{name}'")?;
        }
    } else {
        writeln!(node_type, "    pass")?;
    }

    new_files.add_content(node_path, node_type);

    Ok(())
}
