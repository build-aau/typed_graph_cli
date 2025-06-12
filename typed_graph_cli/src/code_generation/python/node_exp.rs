use build_script_lang::schema::{NodeExp, Schema};
use std::fmt::Write;
use std::path::Path;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToSnakeCase};

use super::{write_comments, write_fields};

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

        writeln!(s, "from ..node_type import NodeType")?;
        writeln!(s, "from ..edge_type import EdgeType")?;
        writeln!(s, "from ..structs import *")?;
        writeln!(s, "from ..types import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from typed_graph import NodeExt, RecievedNoneValue")?;
        writeln!(
            s,
            "from typing import Optional, List, Set, Dict, Iterator, Tuple, ClassVar, TYPE_CHECKING"
        )?;
        writeln!(s, "from pydantic import Field, AliasChoices")?;
        writeln!(s, "")?;
        writeln!(s, "if TYPE_CHECKING:")?;
        writeln!(s, "    from .. import *")?;
        writeln!(s, "    from ..edges import *")?;
        writeln!(s, "    from ..nodes import *")?;
        writeln!(s, "")?;
        writeln!(s, "class {node_name}(NodeExt[NodeId, NodeType]):")?;
        write_comments(&mut s, &self.comments)?;
        writeln!(s, "    id: NodeId")?;
        write_fields(&mut s, &self.fields, false)?;
        writeln!(s, "")?;
        writeln!(s, "    def get_id(self) -> NodeId:")?;
        writeln!(s, "        return self.id")?;
        writeln!(s)?;
        writeln!(s, "    def set_id(self, id: NodeId) -> None:")?;
        writeln!(s, "        self.id = id")?;
        writeln!(s)?;
        writeln!(s, "    def get_type(self) -> NodeType:")?;
        writeln!(s, "        return NodeType.{node_name}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}

/// Write ./nodes.rs
pub(super) fn write_nodes_py<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let node_path = schema_folder.join("node.py");

    let nodes: Vec<_> = schema.nodes().map(|n| n.name.to_string()).collect();

    let mut node = String::new();
    writeln!(node, "from .node_type import NodeType")?;
    writeln!(node, "from .nodes import *")?;
    writeln!(node, "from typed_graph import NestedEnum")?;
    writeln!(node, "from ..imports import *")?;
    writeln!(node, "from .imports import *")?;
    writeln!(node)?;
    writeln!(node, "class Node(NestedEnum):")?;
    if nodes.is_empty() {
        writeln!(node, "    pass")?
    } else {
        for node_type in nodes {
            writeln!(node, "    {node_type} = {node_type}")?;
        }
    }
    writeln!(node)?;
    writeln!(node, "    def get_id(self) -> NodeId:")?;
    writeln!(node, "        ...")?;
    writeln!(node)?;
    writeln!(node, "    def set_id(self, id: NodeId) -> None:")?;
    writeln!(node, "        ...")?;
    writeln!(node)?;
    writeln!(node, "    def get_type(self) -> NodeType:")?;
    writeln!(node, "        ...")?;

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
