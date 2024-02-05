use build_script_lang::schema::{EdgeExp, Schema};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Write};
use std::ops::Bound;
use std::path::Path;

use crate::{targets, CodeGenerator, Direction, GenResult, GeneratedCode, ToPythonType, ToSnakeCase};

impl<I> CodeGenerator<targets::Python> for EdgeExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let edge_name = &self.name;
        let edges_path = p.as_ref().join(format!(
            "{}.py",
            CodeGenerator::<targets::Python>::get_filename(self)
        ));
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
            writeln!(s, "    \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "    {comment}")?;
            }
            writeln!(s, "    \"\"\"")?;
        }
        writeln!(s, "    id: EdgeId")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            let field_type = field_value.field_type.to_python_type();
            writeln!(s, "     {field_name}: {field_type}")?;
            if field_value.comments.has_doc() {
                writeln!(s, "    \"\"\"")?;
                for comment in field_value.comments.iter_doc() {
                    writeln!(s, "    {comment}")?;
                }
                writeln!(s, "    \"\"\"")?;
            }
        }
        writeln!(s, "")?;
        writeln!(s, "    def get_type(self) -> EdgeType:")?;
        writeln!(s, "        return EdgeType.{edge_name}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(edges_path, s);

        Ok(new_files)
    }
}

// Write ./edges.rs
pub(super) fn write_edges_py(new_files: &mut GeneratedCode, schema_folder: &Path) -> GenResult<()> {
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
pub(super) fn write_edge_type_py<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
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

pub(super) fn write_edge_endpoints_py<I: Debug + Ord>(
    schema: &Schema<I>,
    new_file: &mut GeneratedCode,
    nodes_path: &Path
) -> GenResult<()> {
    let schema_name = schema.version.replace(".", "_");

    let mut edges: HashMap<_, HashMap<_, Vec<_>>> = HashMap::new();

    for edge in schema.edges() {
        for ((source, target), endpoint) in &edge.endpoints {
            edges
                .entry(source)
                .or_default()
                .entry(Direction::Forward)
                .or_default()
                .push((endpoint, edge));
            edges
                .entry(target)
                .or_default()
                .entry(Direction::Backwards)
                .or_default()
                .push((endpoint, edge));
        }
    }

    let mut nodes = HashMap::new();

    for node in schema.nodes() {
        nodes.insert(&node.name, node);
    }

    // Foreach node create an impl block with getter functions for adjecent nodes and edges
    for (node, directions) in edges {
        let mut edge_impl: HashSet<_> = HashSet::new();
        let mut endpoint_impl: HashSet<_> = HashSet::new();
        let mut s = String::new();

        // Create getter functions for nodes and edges in a specific direction
        for (dir, endpoints) in directions {
            // Create maps storing how many types of edges go from and to each node type
            // It is important to use a vector as the same edgetype may be used multiple times
            // This allow the differnet functions to figure out if they can be cast to a specific type safely
            let mut grouped_by_end: HashMap<_, HashMap<_, Vec<_>>> = HashMap::new();
            let mut grouped_by_start: HashMap<_, HashMap<_, Vec<_>>> = HashMap::new();

            for (endpoint, edge) in &endpoints {
                let (start, end) = match dir {
                    Direction::Forward => (&endpoint.source, &endpoint.target),
                    Direction::Backwards => (&endpoint.target, &endpoint.source),
                };

                grouped_by_end
                    .entry(dir)
                    .or_default()
                    .entry(end)
                    .or_default()
                    .push(edge);
                grouped_by_start
                    .entry(dir)
                    .or_default()
                    .entry(start)
                    .or_default()
                    .push(&edge.name);
            }

            // Helper values to make the code more direction agnostic
            let search_dir = match dir {
                Direction::Forward => "outgoing",
                Direction::Backwards => "incoming",
            };

            let function_suffix = match dir {
                Direction::Forward => "out",
                Direction::Backwards => "inc",
            };

            let rename_attribute_name = match dir {
                Direction::Forward => "rename_out",
                Direction::Backwards => "rename_inc",
            };

            // Create getter functions with a fixed node type
            for (end, edges) in grouped_by_end.get(&dir).unwrap() {
                let mut edge_types = Vec::new();
                for edge in edges {
                    edge_types.push(format!("EdgeType.{}", edge.name));
                }

                let edge_types_patterns = edge_types.join(", ");

                // If there are no other types of edge to the given node then we can safely cast the edge into the specific one
                let only_edge_type = if edges.len() == 1 {
                    Some(edges.get(0).unwrap())
                } else {
                    None
                };

                let (output_edge_type, is_option) = if let Some(edge) = &only_edge_type {
                    // There may be multiple endpoint to and from the node
                    // So the quantity is calculated as the sum of all endpoints
                    let quantity = edge
                        .endpoints
                        .iter()
                        .filter(|(_, endpoint)| match dir {
                            Direction::Forward => {
                                node == &endpoint.source && end == &&endpoint.target
                            }
                            Direction::Backwards => {
                                node == &endpoint.target && end == &&endpoint.source
                            }
                        })
                        .fold(0, |acc, (_, endpoint)| match endpoint.quantity.quantity {
                            Bound::Included(i) => acc + i,
                            // If the quantity is < 0 then it is the same as <= 0 or acc + 0 = acc
                            Bound::Excluded(i) if i == 0 => acc,
                            Bound::Excluded(i) => acc + i - 1,
                            Bound::Unbounded => acc + 999,
                        });

                    (edge.name.to_string(), quantity == 1)
                } else {
                    ("Edge".to_string(), false)
                };

                let return_type = if is_option {
                    format!("Optional[Tuple['{output_edge_type}', '{end}']]")
                } else {
                    format!(
                        "Iterable[Tuple['{output_edge_type}', '{end}']]"
                    )
                };

                let rename_attribute = nodes.get(node).and_then(|n| {
                    n.attributes
                        .get_functions(rename_attribute_name)
                        .into_iter()
                        .filter_map(|attr| attr.values.get(0).zip(attr.values.get(1)))
                        .find(|(_, rename_node)| rename_node == end)
                });

                let node_func_name = if let Some((new_name, _)) = rename_attribute {
                    new_name.to_snake_case()
                } else {
                    let function_name = end.to_snake_case();
                    format!("{function_name}_{function_suffix}")
                };

                // Write get by node type method
                writeln!(s, "")?;
                writeln!(s, "    def get_{node_func_name}(self, g: '{schema_name}Graph') -> {return_type}:")?;
                writeln!(s, "        edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() in [{edge_types_patterns}])")?;
                writeln!(s, "        nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                writeln!(s, "        nodes = filter(lambda ne: ne[1].get_type() == NodeType.{end}, nodes)")?;
                if is_option {
                    writeln!(s, "        nodes = next(nodes)")?;
                }
                writeln!(s, "        return nodes")?;
            }

            // Create getter functions with a fixed edge type
            for (endpoint, edge) in &endpoints {
                let edge_type = &edge.name;

                let edge_func_name =
                    if let Some(new_name) = edge.attributes.get_key_value(rename_attribute_name) {
                        new_name.value.to_snake_case()
                    } else {
                        let function_name = edge.name.to_snake_case();
                        format!("{function_name}_{function_suffix}")
                    };

                let target_type = match dir {
                    Direction::Forward => &endpoint.target,
                    Direction::Backwards => &endpoint.source,
                };

                let source_type = match dir {
                    Direction::Forward => &endpoint.source,
                    Direction::Backwards => &endpoint.target,
                };

                // If there are multiple of the same edge type to a node there should only be one function implementation
                if edge_impl.contains(&(edge.name.clone(), dir)) {
                    continue;
                } else {
                    edge_impl.insert((edge.name.clone(), dir));
                }

                // Check if there exists other edges of the same type frome the start of this node
                let edges_of_type = grouped_by_start
                    .entry(dir)
                    .or_default()
                    .get(source_type)
                    .unwrap()
                    .iter()
                    .filter(|name| name == &&&edge.name)
                    .count();

                // If there are other edges we cannot cast the node to a specific type
                // And the must be more
                if edges_of_type > 1 {
                    // Write get by edge type method
                    writeln!(s, "")?;
                    writeln!(s, "    def get_{edge_func_name}(self, g: '{schema_name}Graph') -> Iterable[Tuple['{edge_type}', 'Node']]:")?;
                    writeln!(s, "       edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                    writeln!(s, "       nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                    writeln!(s, "       return nodes")?;

                // If there are no other edges we can safely cast the node to a specific type
                } else {
                    let is_option = endpoint.quantity.quantity == Bound::Included(1)
                        || endpoint.quantity.quantity == Bound::Excluded(2);
                    let return_type = if is_option {
                        format!("Optional[Tuple['{edge_type}', '{target_type}']]")
                    } else {
                        format!("Iterable[Tuple['{edge_type}', '{target_type}']]")
                    };

                    // Write get by edge type method
                    writeln!(s, "")?;
                    writeln!(s, "    def get_{edge_func_name}(self, g: '{schema_name}Graph') -> {return_type}:")?;
                    writeln!(s, "       edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                    writeln!(s, "       nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                    if is_option {
                        writeln!(s, "       nodes = next(nodes)")?;
                    }
                    writeln!(s, "       return nodes")?;
                }
            }

            // Create getter functions with a fixed node and edge type
            for (endpoint, edge) in &endpoints {
                let edge_type = &edge.name;

                let target_type = match dir {
                    Direction::Forward => &endpoint.target,
                    Direction::Backwards => &endpoint.source,
                };

                let edge_func_name = if let Some(new_name) =
                    endpoint.attributes.get_key_value(rename_attribute_name)
                {
                    new_name.value.to_snake_case()
                } else {
                    let edge_name = edge.name.to_snake_case();
                    let target_name = target_type.to_snake_case();
                    format!("{target_name}_via_{edge_name}_{function_suffix}")
                };

                // If there are multiple of the same edge type to a node there should only be one function implementation
                if endpoint_impl.contains(&(
                    edge.name.to_string(),
                    endpoint.source.to_string(),
                    endpoint.target.to_string(),
                    dir,
                )) {
                    continue;
                } else {
                    endpoint_impl.insert((
                        edge.name.to_string(),
                        endpoint.source.to_string(),
                        endpoint.target.to_string(),
                        dir,
                    ));
                }

                // Check if the result can be
                let is_option = endpoint.quantity.quantity == Bound::Included(1)
                    || endpoint.quantity.quantity == Bound::Excluded(2);
                let return_type = if is_option {
                    format!("Optional[Tuple['{edge_type}', '{target_type}']]")
                } else {
                    format!(
                        "Iterable[Tuple['{edge_type}', '{target_type}']]"
                    )
                };

                // Write get by edge type method
                writeln!(s, "")?;
                writeln!(s, "    def get_{edge_func_name}(self, g: '{schema_name}Graph') -> {return_type}:")?;
                writeln!(s, "       edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                writeln!(s, "       nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                writeln!(s, "       nodes = filter(lambda ne: ne[1].get_type() == NodeType.{target_type}, nodes)")?;
                if is_option {
                    writeln!(s, "       nodes = next(nodes)")?;
                }
                writeln!(s, "       return nodes")?;
            }
        }

        let node_path = nodes_path.join(format!("{}.py", node.to_snake_case()));
        new_file.add_content(node_path, s);
    }

    Ok(())
}
