use build_script_lang::schema::{EdgeExp, LowerBound, Schema};
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Write};
use std::path::Path;

use crate::{
    targets, CodeGenerator, Direction, GenResult, GeneratedCode, ToSnakeCase,
};

use super::{write_comments, write_fields};

enum EdgeRepresentation {
    Result,
    Option,
    Iterator
}

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
        writeln!(s, "from ..structs import *")?;
        writeln!(s, "from ..types import *")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s, "from pydantic import Field, AliasChoices")?;
        writeln!(s, "from typing import Optional, List, Dict, ClassVar")?;
        writeln!(s, "from typed_graph import EdgeExt")?;
        writeln!(s, "")?;
        writeln!(s, "class {edge_name}(EdgeExt[EdgeId, EdgeType]):")?;
        write_comments(&mut s, &self.comments)?;
        writeln!(s, "    tagging: ClassVar[bool] = False")?;
        writeln!(s, "    id: EdgeId")?;
        write_fields(&mut s, &self.fields)?;
        writeln!(s)?;
        writeln!(s, "    def get_id(self) -> EdgeId:")?;
        writeln!(s, "        return self.id")?;
        writeln!(s)?;
        writeln!(s, "    def set_id(self, id: EdgeId) -> None:")?;
        writeln!(s, "        self.id = id")?;
        writeln!(s, "")?;
        writeln!(s, "    def get_type(self) -> EdgeType:")?;
        writeln!(s, "        return EdgeType.{edge_name}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(edges_path, s);

        Ok(new_files)
    }
}

// Write ./edges.rs
pub(super) fn write_edges_py<I: Ord>(schema: &Schema<I>, new_files: &mut GeneratedCode, schema_folder: &Path) -> GenResult<()> {
    let edge_path = schema_folder.join("edge.py");

    let edges: Vec<_> = schema.edges()
        .map(|n| n.name.to_string())
        .collect();

    let mut edge = String::new();
    writeln!(edge, "from typed_graph import NestedEnum")?;
    writeln!(edge, "from .. import imports")?;
    writeln!(edge, "from .edges import *")?;
    writeln!(edge, "")?;
    writeln!(edge, "class Edge(NestedEnum):")?;
    if edges.is_empty() {
        writeln!(edge, "    pass")?
    } else {
        for edge_type in edges {
            writeln!(edge, "    {edge_type}: {edge_type}")?;
        }
    }

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
    nodes_path: &Path,
) -> GenResult<()> {
    let schema_name = schema.version.replace(".", "_");

    let mut edges: BTreeMap<_, BTreeMap<_, Vec<_>>> = BTreeMap::new();

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

    let mut nodes = BTreeMap::new();

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
            let mut grouped_by_end: BTreeMap<_, BTreeMap<_, Vec<_>>> = BTreeMap::new();
            let mut grouped_by_start: BTreeMap<_, BTreeMap<_, Vec<_>>> = BTreeMap::new();

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

                let mut edge_repr = EdgeRepresentation::Result;
                
                for edge in edges {
                    edge_repr = edge
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
                        .map(|(_, endpoint)| match dir {
                            Direction::Forward => &endpoint.outgoing_quantity,
                            Direction::Backwards => &endpoint.incoming_quantity,
                        })
                        .fold(edge_repr, |repr, quantity| match quantity.bounds {
                            Some((lower, upper)) => match repr {
                                EdgeRepresentation::Result => if lower == LowerBound::One && upper == 1 {
                                    EdgeRepresentation::Result
                                } else {
                                    if upper == 1 {
                                        EdgeRepresentation::Option
                                    } else {
                                        EdgeRepresentation::Iterator
                                    }
                                }
                                EdgeRepresentation::Option => if upper == 1 {
                                    EdgeRepresentation::Option
                                } else {
                                    EdgeRepresentation::Iterator
                                }
                                EdgeRepresentation::Iterator => EdgeRepresentation::Iterator
                            },
                            None => EdgeRepresentation::Iterator,
                        });
                }

                let output_edge_type = only_edge_type.map_or_else(|| "Edge".to_string(), |edge| edge.name.to_string());

                let return_type = match edge_repr {
                    EdgeRepresentation::Result => format!("Tuple['{output_edge_type}', '{end}']"),
                    EdgeRepresentation::Option => format!("Optional[Tuple['{output_edge_type}', '{end}']]"),
                    EdgeRepresentation::Iterator => format!("Iterable[Tuple['{output_edge_type}', '{end}']]"),
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
                writeln!(
                    s,
                    "    def {node_func_name}(self, g: '{schema_name}Graph') -> {return_type}:"
                )?;
                writeln!(s, "        edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() in [{edge_types_patterns}])")?;
                writeln!(
                    s,
                    "        nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)"
                )?;
                writeln!(
                    s,
                    "        nodes = filter(lambda ne: ne[1].get_type() == NodeType.{end}, nodes)"
                )?;
                match edge_repr {
                    EdgeRepresentation::Iterator => (),
                    EdgeRepresentation::Option => writeln!(s, "        nodes = next(nodes, None)")?,
                    EdgeRepresentation::Result => {
                        writeln!(s, "        nodes = next(nodes, None)")?;
                        writeln!(s, "        if nodes is None:")?;
                        writeln!(s, "            raise RecievedNoneValue(f'{{self.get_id()}}({{self.get_type()}})', '{node_func_name}')")?;
                    },
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
                    writeln!(s, "        edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                    writeln!(s, "        nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                    writeln!(s, "        return nodes")?;

                // If there are no other edges we can safely cast the node to a specific type
                } else {
                    let quantity = match dir {
                        Direction::Forward => &endpoint.outgoing_quantity.bounds,
                        Direction::Backwards => &endpoint.incoming_quantity.bounds,
                    };

                    let edge_repr = match quantity {
                        None => EdgeRepresentation::Iterator,
                        Some((lower, upper)) => match lower {
                            LowerBound::Zero => if *upper == 1 {
                                EdgeRepresentation::Option
                            } else {
                                EdgeRepresentation::Iterator
                            },
                            LowerBound::One => if *upper == 1 {
                                EdgeRepresentation::Result
                            } else {
                                EdgeRepresentation::Iterator
                            }
                        }
                    };

                    let return_type = match edge_repr {
                        EdgeRepresentation::Iterator => format!("Iterable[Tuple['{edge_type}', '{target_type}']]"),
                        EdgeRepresentation::Option => format!("Optional[Tuple['{edge_type}', '{target_type}']]"),
                        EdgeRepresentation::Result => format!("Tuple['{edge_type}', '{target_type}']")
                    };

                    // Write get by edge type method
                    writeln!(s, "")?;
                    writeln!(s, "    def {edge_func_name}(self, g: '{schema_name}Graph') -> {return_type}:")?;
                    writeln!(s, "        edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                    writeln!(s, "        nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)")?;
                    match edge_repr {
                        EdgeRepresentation::Iterator => (),
                        EdgeRepresentation::Option => writeln!(s, "        nodes = next(nodes, None)")?,
                        EdgeRepresentation::Result => {
                            writeln!(s, "        nodes = next(nodes, None)")?;
                            writeln!(s, "        if nodes is None:")?;
                            writeln!(s, "            raise RecievedNoneValue(f'{{self.get_id()}}({{self.get_type()}})', '{edge_func_name}')")?;
                        },
                    }
                    writeln!(s, "        return nodes")?;
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
                
                let quantity = match dir {
                    Direction::Forward => &endpoint.outgoing_quantity.bounds,
                    Direction::Backwards => &endpoint.incoming_quantity.bounds,
                };

                let edge_repr = match quantity {
                    None => EdgeRepresentation::Iterator,
                    Some((lower, upper)) => match lower {
                        LowerBound::Zero => if *upper == 1 {
                            EdgeRepresentation::Option
                        } else {
                            EdgeRepresentation::Iterator
                        },
                        LowerBound::One => if *upper == 1 {
                            EdgeRepresentation::Result
                        } else {
                            EdgeRepresentation::Iterator
                        }
                    }
                };

                let return_type = match edge_repr {
                    EdgeRepresentation::Iterator => format!("Iterable[Tuple['{edge_type}', '{target_type}']]"),
                    EdgeRepresentation::Option => format!("Optional[Tuple['{edge_type}', '{target_type}']]"),
                    EdgeRepresentation::Result => format!("Tuple['{edge_type}', '{target_type}']")
                };

                // Write get by edge type method
                writeln!(s, "")?;
                writeln!(
                    s,
                    "    def {edge_func_name}(self, g: '{schema_name}Graph') -> {return_type}:"
                )?;
                writeln!(s, "        edges = g.get_{search_dir}_filter(self.get_id(), lambda e: e.get_type() == EdgeType.{edge_type})")?;
                writeln!(
                    s,
                    "        nodes = map(lambda e: (e.weight, g.get_node(e.get_outer())), edges)"
                )?;
                writeln!(s, "        nodes = filter(lambda ne: ne[1].get_type() == NodeType.{target_type}, nodes)")?;
                match edge_repr {
                    EdgeRepresentation::Iterator => (),
                    EdgeRepresentation::Option => writeln!(s, "        nodes = next(nodes, None)")?,
                    EdgeRepresentation::Result => {
                        writeln!(s, "        nodes = next(nodes, None)")?;
                        writeln!(s, "        if nodes is None:")?;
                        writeln!(s, "            raise RecievedNoneValue(f'{{self.get_id()}}({{self.get_type()}})', '{edge_func_name}')")?;
                    },
                }
                writeln!(s, "        return nodes")?;
            }
        }

        let node_path = nodes_path.join(format!("{}.py", node.to_snake_case()));
        new_file.add_content(node_path, s);
    }

    Ok(())
}
