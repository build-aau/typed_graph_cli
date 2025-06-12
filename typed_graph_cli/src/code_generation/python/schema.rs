use crate::*;
use build_script_lang::schema::{Schema, SchemaStm};
use std::fmt::{Debug, Write};
use std::path::Path;

use super::{
    write_edge_endpoints_py, write_edge_type_py, write_edges_py, write_node_type_py, write_nodes_py,
};
use std::fs::create_dir;

impl<I> CodeGenerator<targets::Python> for Schema<I>
where
    I: Ord + Debug,
{
    fn get_filename(&self) -> String {
        self.version.to_string().replace(".", "_").to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let schema_folder = p
            .as_ref()
            .join(CodeGenerator::<targets::Python>::get_filename(self));
        if !schema_folder.exists() {
            create_dir(&schema_folder)?;
        }

        let nodes_folder = schema_folder.join("nodes");
        let structs_folder = schema_folder.join("structs");
        let edges_folder = schema_folder.join("edges");
        let types_folder = schema_folder.join("types");

        if !nodes_folder.exists() {
            create_dir(&nodes_folder)?;
        }
        if !structs_folder.exists() {
            create_dir(&structs_folder)?;
        }
        if !edges_folder.exists() {
            create_dir(&edges_folder)?;
        }
        if !types_folder.exists() {
            create_dir(&types_folder)?;
        }

        let mut new_files = GeneratedCode::new();

        write_content(
            self,
            &mut new_files,
            &nodes_folder,
            &structs_folder,
            &edges_folder,
            &types_folder,
        )?;
        write_nodes_py(self, &mut new_files, &schema_folder)?;
        write_node_type_py(self, &mut new_files, &schema_folder)?;
        write_edges_py(self, &mut new_files, &schema_folder)?;
        write_edge_type_py(self, &mut new_files, &schema_folder)?;
        write_init(
            self,
            &mut new_files,
            &schema_folder,
            &nodes_folder,
            &structs_folder,
            &edges_folder,
            &types_folder,
        )?;
        write_schema_impl_py(self, &mut new_files, &schema_folder)?;
        write_edge_endpoints_py(self, &mut new_files, &nodes_folder)?;

        Ok(new_files)
    }
}

fn write_content<I>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    nodes_folder: &Path,
    structs_folder: &Path,
    edges_folder: &Path,
    types_folder: &Path,
) -> GenResult<()>
where
    I: Ord,
{
    // Write ./{nodes|edges|types}/{filename}.rs
    for stm in schema.iter() {
        let added_files = match stm {
            SchemaStm::Node(n) => {
                CodeGenerator::<targets::Python>::aggregate_content(n, &nodes_folder)
            }
            SchemaStm::Struct(n) => {
                CodeGenerator::<targets::Python>::aggregate_content(n, &structs_folder)
            }
            SchemaStm::Edge(n) => {
                CodeGenerator::<targets::Python>::aggregate_content(n, &edges_folder)
            }
            SchemaStm::Enum(n) => {
                CodeGenerator::<targets::Python>::aggregate_content(n, &types_folder)
            }
            SchemaStm::Import(_) => Ok(GeneratedCode::new()),
        }?;

        new_files.append(added_files);
    }

    Ok(())
}

fn write_init<I>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
    nodes_folder: &Path,
    structs_folder: &Path,
    edges_folder: &Path,
    types_folder: &Path,
) -> GenResult<()>
where
    I: Ord,
{
    // Write ./{nodes|edges|types}/__init__.py
    let nodes_init_path = nodes_folder.join("__init__.py");
    let structs_init_path = structs_folder.join("__init__.py");
    let edges_init_path = edges_folder.join("__init__.py");
    let types_init_path = types_folder.join("__init__.py");

    let mut nodes_init = String::new();
    let mut structs_init = String::new();
    let mut edges_init = String::new();
    let mut types_init = String::new();

    for stm in schema.iter() {
        let (filename, type_name, f) = match stm {
            SchemaStm::Node(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &n.name,
                &mut nodes_init,
            ),
            SchemaStm::Struct(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &n.name,
                &mut structs_init,
            ),
            SchemaStm::Edge(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &n.name,
                &mut edges_init,
            ),
            SchemaStm::Enum(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &n.name,
                &mut types_init,
            ),
            SchemaStm::Import(_) => continue,
        };

        writeln!(f, "from .{filename} import {type_name}")?;
    }

    writeln!(nodes_init, "__all__ = [")?;
    writeln!(structs_init, "__all__ = [")?;
    writeln!(edges_init, "__all__ = [")?;
    writeln!(types_init, "__all__ = [")?;

    for stm in schema.iter() {
        let (type_name, f) = match stm {
            SchemaStm::Node(n) => (&n.name, &mut nodes_init),
            SchemaStm::Struct(n) => (&n.name, &mut structs_init),
            SchemaStm::Edge(n) => (&n.name, &mut edges_init),
            SchemaStm::Enum(n) => (&n.name, &mut types_init),
            SchemaStm::Import(_) => continue,
        };

        writeln!(f, "    '{type_name}',")?;
    }

    writeln!(nodes_init, "]")?;
    writeln!(structs_init, "]")?;
    writeln!(edges_init, "]")?;
    writeln!(types_init, "]")?;

    new_files.add_content(nodes_init_path, nodes_init);
    new_files.add_content(structs_init_path, structs_init);
    new_files.add_content(edges_init_path, edges_init);
    new_files.add_content(types_init_path, types_init);

    // Write ./mod.rs
    let schema_init_path = schema_folder.join("__init__.py");

    let schema_name = schema.version.replace(".", "_");
    let mut s = String::new();
    writeln!(s, "from .edge_type import EdgeType")?;
    writeln!(s, "from .node_type import NodeType")?;
    writeln!(s, "")?;
    // Resolve circular references
    writeln!(s, "from .imports import *")?;
    writeln!(s, "from ..imports import *")?;
    writeln!(s, "from .structs import *")?;
    writeln!(s, "from .types import *")?;
    writeln!(s, "")?;
    for stm in schema.iter() {
        match stm {
            SchemaStm::Struct(n) if n.has_external_ref() => writeln!(s, "{}.model_rebuild()", n.name)?,
            SchemaStm::Enum(n) if n.has_external_ref() => writeln!(s, "{}.model_rebuild()", n.name)?,
            _ => ()
        }
    }
    
    writeln!(s, "")?;
    writeln!(s, "from .edge import Edge")?;
    writeln!(s, "from .node import Node")?;
    writeln!(s, "from .schema import {schema_name}")?;
    writeln!(s, "")?;
    writeln!(s, "from typed_graph import TypedGraph, PartialTypedGraph")?;
    writeln!(s, "")?;
    writeln!(s, "from .import imports as imports1")?;
    writeln!(s, "from ..import imports as imports2")?;
    writeln!(s, "")?;
    writeln!(s, "if hasattr(imports1, 'NodeId'):")?;
    writeln!(s, "    NodeId = imports1.NodeId")?;
    writeln!(s, "else:")?;
    writeln!(s, "    NodeId = imports2.NodeId")?;
    writeln!(s, "")?;
    writeln!(s, "if hasattr(imports1, 'EdgeId'):")?;
    writeln!(s, "    EdgeId = imports1.EdgeId")?;
    writeln!(s, "else:")?;
    writeln!(s, "    EdgeId = imports2.EdgeId")?;
    writeln!(s, "")?;
    writeln!(s, "{schema_name}Graph = TypedGraph[Node, Edge, NodeId, EdgeId, NodeType, EdgeType, {schema_name}]")?;
    writeln!(s, "{schema_name}PartialGraph = PartialTypedGraph[Node, Edge, NodeId, EdgeId, NodeType, EdgeType, {schema_name}]")?;
    writeln!(s, "")?;
    writeln!(s, "__all__ = [")?;
    writeln!(s, "    'EdgeType',")?;
    writeln!(s, "    'NodeType',")?;
    writeln!(s, "    'Edge',")?;
    writeln!(s, "    'Node',")?;
    writeln!(s, "    '{schema_name}Graph',")?;
    writeln!(s, "    '{schema_name}PartialGraph',")?;
    writeln!(s, "    '{schema_name}',")?;
    writeln!(s, "]")?;

    new_files.add_content(schema_init_path, s);

    let imports_path = schema_folder.join("imports.py");
    let mut s = String::new();
    writeln!(s, "from uuid import UUID")?;
    writeln!(s, "")?;
    writeln!(s, "NodeId = UUID")?;
    writeln!(s, "EdgeId = UUID")?;

    new_files.create_file_with_default(imports_path, s);

    Ok(())
}

fn write_schema_impl_py<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let schema_path = schema_folder.join("schema.py");
    let schema_name = schema.version.replace(".", "_");
    let schema_version = &schema.version;

    let mut schema_py = String::new();

    writeln!(schema_py, "from typing import TypeVar")?;
    writeln!(
        schema_py,
        "from typed_graph import SchemaExt, TypeStatus, RustRootModel"
    )?;
    writeln!(schema_py, "from .node import Node")?;
    writeln!(schema_py, "from .edge import Edge")?;
    writeln!(schema_py, "from .edge_type import EdgeType")?;
    writeln!(schema_py, "from .node_type import NodeType")?;
    writeln!(
        schema_py,
        "from typing import ClassVar, Dict, Tuple, Literal"
    )?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "NK = TypeVar('NK')")?;
    writeln!(schema_py, "EK = TypeVar('EK')")?;
    writeln!(
        schema_py,
        "class {schema_name}(SchemaExt[Node, Edge, NK, EK, NodeType, EdgeType], RustRootModel):"
    )?;
    writeln!(
        schema_py,
        "    endpoint_meta: ClassVar[Dict[Tuple[EdgeType, NodeType, NodeType], int]] = {{"
    )?;
    for e in schema.edges() {
        let edge_type = &e.name;
        for ((source, target), endpoint) in &e.endpoints {
            let outgoing_quantity = endpoint.outgoing_quantity.bounds;
            let incoming_quantity = endpoint.incoming_quantity.bounds;

            let outgoing_quantity_text =
                outgoing_quantity.map_or_else(|| "None".to_string(), |(_, q)| q.to_string());
            let incoming_quantity_text =
                incoming_quantity.map_or_else(|| "None".to_string(), |(_, q)| q.to_string());

            writeln!(schema_py, "        (EdgeType.{edge_type}, NodeType.{source}, NodeType.{target}): ({outgoing_quantity_text}, {incoming_quantity_text}),")?;
        }
    }
    writeln!(schema_py, "    }}")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "    tagging: ClassVar[bool] = False")?;
    writeln!(schema_py, "    root: Literal['{schema_version}']")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "    def name(*args) -> str:")?;
    writeln!(schema_py, "        return '{schema_version}'")?;
    writeln!(schema_py, "")?;
    writeln!(
        schema_py,
        "    def allow_node(self, node_type: NodeType) -> TypeStatus | bool:"
    )?;
    writeln!(schema_py, "        return isinstance(node_type, NodeType) and node_type in NodeType.__members__.keys()")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "    def allow_edge(self, outgoing_quantity: int, incoming_quantity: int, edge_type: EdgeType, source_type: NodeType, target_type: NodeType) -> TypeStatus | bool:")?;
    writeln!(schema_py, "        source_allowed = isinstance(source_type, NodeType) and source_type in NodeType.__members__.keys()")?;
    writeln!(schema_py, "        target_allowed = isinstance(target_type, NodeType) and target_type in NodeType.__members__.keys()")?;
    writeln!(schema_py, "        edge_allowed = isinstance(edge_type, EdgeType) and edge_type in EdgeType.__members__.keys()")?;
    writeln!(
        schema_py,
        "        edge_meta_key = (edge_type, source_type, target_type)"
    )?;
    writeln!(
        schema_py,
        "        endpoint_allowed = edge_meta_key in {schema_name}.endpoint_meta"
    )?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "        if not source_allowed or not target_allowed or not edge_allowed or not endpoint_allowed:")?;
    writeln!(schema_py, "            return TypeStatus.InvalidType")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "        (outgoing_allowed_quantity, incoming_allowed_quantity) = {schema_name}.endpoint_meta[(edge_type, source_type, target_type)]")?;
    writeln!(schema_py, "        if outgoing_allowed_quantity is not None and outgoing_quantity > outgoing_allowed_quantity:")?;
    writeln!(schema_py, "            return TypeStatus.ToManyOutgoing")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "        if incoming_allowed_quantity is not None and incoming_quantity > incoming_allowed_quantity:")?;
    writeln!(schema_py, "            return TypeStatus.ToManyIncoming")?;
    writeln!(schema_py, "")?;
    writeln!(schema_py, "        return TypeStatus.Ok")?;
    writeln!(schema_py, "")?;

    new_files.add_content(schema_path, schema_py);
    Ok(())
}
