use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{EdgeExp, Schema, Visibility};
use build_script_shared::{InputMarker, InputType};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Write};
use std::ops::Bound;
use std::path::Path;

use crate::{targets, CodeGenerator, Direction, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

impl<I> CodeGenerator<targets::Rust> for EdgeExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let edge_name = &self.name;
        let edges_path = p.as_ref().join(format!(
            "{}.rs",
            CodeGenerator::<targets::Rust>::get_filename(self)
        ));
        let mut s = String::new();

        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::super::imports::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::imports::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use std::collections::HashMap;")?;
        writeln!(s, "use typed_graph::*;")?;
        writeln!(s, "use serde::{{Serialize, Deserialize}};")?;
        #[cfg(feature = "diff")]
        writeln!(s, "use changesets::Changeset;")?;

        let attributes = vec![
            "Clone".to_string(),
            "Debug".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            #[cfg(feature = "diff")]
            "Changeset".to_string(),
        ];
        let attribute_s = attributes.join(", ");

        writeln!(s, "")?;
        for comment in self.comments.iter_doc() {
            writeln!(s, "/// {comment}")?;
        }
        writeln!(s, "#[derive({attribute_s})]")?;
        writeln!(s, "pub struct {edge_name}<EK> {{")?;
        writeln!(s, "    pub(crate) id: EK,")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            for comment in field_value.comments.iter_doc() {
                writeln!(s, "    /// {comment}")?;
            }
            let vis = match field_value.visibility {
                Visibility::Local => "pub(crate) ",
                Visibility::Public => "pub ",
            };
            let field_type = field_value.field_type.to_rust_type();
            writeln!(s, "    {vis}{field_name}: {field_type},")?;
        }
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "impl<EK> {edge_name}<EK> {{")?;
        writeln!(s, "    pub fn new(")?;
        write!(s, "       id: EK")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            writeln!(s, ",")?;
            let field_type = field_value.field_type.to_rust_type();
            write!(s, "       {field_name}: {field_type}")?;
        }
        writeln!(s, "")?;
        writeln!(s, "   ) -> Self {{")?;
        writeln!(s, "        Self {{")?;
        write!(s, "           id")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            writeln!(s, ",")?;
            write!(s, "           {field_name}")?;
        }
        writeln!(s, "")?;
        writeln!(s, "        }}")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "impl<EK> Typed for {edge_name}<EK> {{")?;
        writeln!(s, "    type Type = EdgeType;")?;
        writeln!(s, "    fn get_type(&self) -> EdgeType {{")?;
        writeln!(s, "       EdgeType::{edge_name}")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;
        writeln!(s, "")?;
        writeln!(s, "impl<EK: Key> Id<EK> for {edge_name}<EK> {{")?;
        writeln!(s, "    fn get_id(&self) -> EK {{")?;
        writeln!(s, "       self.id")?;
        writeln!(s, "    }}")?;
        writeln!(s, "")?;
        writeln!(s, "    fn set_id(&mut self, id: EK) {{")?;
        writeln!(s, "        self.id = id")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;
        let name_type = self.fields.get_field("name");

        if let Some(field_value) = name_type {
            let field_type = &field_value.field_type;
            writeln!(s, "")?;
            writeln!(s, "impl<EK> Name for {edge_name}<EK> {{")?;
            writeln!(s, "    type Name = {field_type};")?;
            writeln!(s, "    fn get_name(&self) -> Option<&Self::Name> {{")?;
            writeln!(s, "       Some(&self.name)")?;
            writeln!(s, "    }}")?;
            writeln!(s, "}}")?;
        }
        writeln!(s, "")?;
        writeln!(
            s,
            "impl<EK: std::fmt::Display + Key> std::fmt::Display for {edge_name}<EK> {{"
        )?;
        writeln!(
            s,
            "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
        )?;
        writeln!(
            s,
            "        write!(f, \"{{}}({{}})\", self.get_type(), self.get_id())"
        )?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(edges_path, s);

        Ok(new_files)
    }
}

// Write ./edges.rs
pub(super) fn write_edges_rs<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let edge_path = schema_folder.join("edge.rs");

    let mut edge = String::new();
    let edges: Vec<_> = schema.edges().collect();

    writeln!(edge, "use super::*;")?;
    writeln!(edge, "use std::fmt::Debug;")?;
    writeln!(edge, "use typed_graph::*;")?;
    writeln!(edge, "use serde::{{Serialize, Deserialize}};")?;
    #[cfg(feature = "diff")]
    writeln!(edge, "use changesets::Changeset;")?;

    let attributes = vec![
        "Clone".to_string(),
        "Debug".to_string(),
        "Serialize".to_string(),
        "Deserialize".to_string(),
        #[cfg(feature = "diff")]
        "Changeset".to_string(),
    ];
    let attribute_s = attributes.join(", ");

    writeln!(edge, "")?;
    writeln!(edge, "#[derive({attribute_s})]")?;
    if edges.is_empty() {
        writeln!(edge, "pub struct Edge<EK> {{")?;
        writeln!(edge, "    id: EK")?;
        writeln!(edge, "}}")?;
    } else {
        writeln!(edge, "pub enum Edge<EK> {{")?;
        for e in &edges {
            let edge_type = &e.name;
            writeln!(edge, "    {edge_type}({edge_type}<EK>),")?;
        }
        writeln!(edge, "}}")?;
    }
    writeln!(edge, "")?;
    writeln!(edge, "impl<EK: Key> EdgeExt<EK> for Edge<EK> {{}}")?;

    writeln!(edge, "")?;
    writeln!(edge, "impl<EK> Typed for Edge<EK> {{")?;
    writeln!(edge, "    type Type = EdgeType;")?;
    writeln!(edge, "    fn get_type(&self) -> EdgeType {{")?;
    if !edges.is_empty() {
        writeln!(edge, "        match self {{")?;
        for e in &edges {
            let edge_type = &e.name;
            writeln!(
                edge,
                "            Edge::{edge_type}(_) => EdgeType::{edge_type},"
            )?;
        }
        writeln!(edge, "        }}")?;
    } else {
        writeln!(edge, "            EdgeType")?;
    }
    writeln!(edge, "    }}")?;
    writeln!(edge, "}}")?;

    writeln!(edge, "")?;
    writeln!(edge, "impl<EK: Key> Id<EK> for Edge<EK> {{")?;
    writeln!(edge, "    fn get_id(&self) -> EK {{")?;
    if !edges.is_empty() {
        writeln!(edge, "        match self {{")?;
        for e in &edges {
            let edge_type = &e.name;
            writeln!(edge, "            Edge::{edge_type}(e) => e.get_id(),")?;
        }
        writeln!(edge, "        }}")?;
    } else {
        writeln!(edge, "        self.id")?;
    }
    writeln!(edge, "    }}")?;

    writeln!(edge, "")?;
    writeln!(edge, "    fn set_id(&mut self, id: EK) {{")?;
    if !edges.is_empty() {
        writeln!(edge, "        match self {{")?;
        for e in &edges {
            let edge_type = &e.name;
            writeln!(edge, "            Edge::{edge_type}(e) => e.set_id(id),")?;
        }
        writeln!(edge, "        }}")?;
    } else {
        writeln!(edge, "        self.id = id;")?;
    }
    writeln!(edge, "    }}")?;
    writeln!(edge, "}}")?;

    /* This us up for revew

    let name_type = edges
        .iter()
        .filter_map(|e| e
            .fields
            .get_field("name"))
            .map(|(_, field_value)| field_value.ty.to_string()
        )
        .next();

    if let Some(name_type) = name_type {
        writeln!(edge, "")?;
        writeln!(edge, "impl<EK> Name for Edge<EK> {{")?;
        writeln!(edge, "    type Name = {name_type};")?;
        writeln!(edge, "    fn get_name(&self) -> Option<&Self::Name> {{")?;
        writeln!(edge, "       match self {{")?;
        for e in &edges {
            let edge_type = &e.name;

            if e.fields.has_field("name") {
                writeln!(edge, "        Edge::{edge_type}(e) => e.get_name()")?;
            } else {
                writeln!(edge, "        Edge::{edge_type}(e) => None")?;
            }
        }
        writeln!(edge, "       }}")?;
        writeln!(edge, "    }}")?;
        writeln!(edge, "}}")?;
    }

     */

    for n in &edges {
        let name = n.name.to_string();
        writeln!(edge, "")?;
        writeln!(edge, "impl<EK> From<{name}<EK>> for Edge<EK> {{")?;
        writeln!(edge, "    fn from(other: {name}<EK>) -> Self {{")?;
        writeln!(edge, "       Self::{name}(other)")?;
        writeln!(edge, "    }}")?;
        writeln!(edge, "}}")?;
    }

    for e in &edges {
        let edge_type = &e.name;

        writeln!(edge, "")?;
        writeln!(
            edge,
            "impl<'a, NK, EK, S> Downcast<'a, NK, EK, &'a {edge_type}<EK>, S> for Edge<EK>"
        )?;
        writeln!(edge, "where")?;
        writeln!(edge, "    NK: Key,")?;
        writeln!(edge, "    EK: Key,")?;
        writeln!(edge, "    S: SchemaExt<NK, EK, E = Edge<EK>>")?;
        writeln!(edge, "{{")?;
        writeln!(
            edge,
            "    fn downcast(&'a self) -> SchemaResult<&'a {edge_type}<EK>, NK, EK, S> {{"
        )?;
        writeln!(edge, "        match self {{")?;
        writeln!(edge, "            Edge::{edge_type}(e) => Ok(e),")?;
        writeln!(edge, "            #[allow(unreachable_patterns)]")?;
        writeln!(edge, "            e => Err(TypedError::DownCastFailed(\"{edge_type}\".to_string(), e.get_type().to_string()))")?;
        writeln!(edge, "        }}")?;
        writeln!(edge, "    }}")?;
        writeln!(edge, "}}")?;
    }

    for e in &edges {
        let edge_type = &e.name;

        writeln!(edge, "")?;
        writeln!(
            edge,
            "impl<'a, NK, EK, S> DowncastMut<'a, NK, EK, &'a mut {edge_type}<EK>, S> for Edge<EK>"
        )?;
        writeln!(edge, "where")?;
        writeln!(edge, "    NK: Key,")?;
        writeln!(edge, "    EK: Key,")?;
        writeln!(edge, "    S: SchemaExt<NK, EK, E = Edge<EK>>")?;
        writeln!(edge, "{{")?;
        writeln!(
            edge,
            "    fn downcast_mut(&'a mut self) -> SchemaResult<&'a mut {edge_type}<EK>, NK, EK, S> {{"
        )?;
        writeln!(edge, "        match self {{")?;
        writeln!(edge, "            Edge::{edge_type}(e) => Ok(e),")?;
        writeln!(edge, "            #[allow(unreachable_patterns)]")?;
        writeln!(edge, "            e => Err(TypedError::DownCastFailed(\"{edge_type}\".to_string(), e.get_type().to_string()))")?;
        writeln!(edge, "        }}")?;
        writeln!(edge, "    }}")?;
        writeln!(edge, "}}")?;
    }

    writeln!(edge, "")?;
    writeln!(
        edge,
        "impl<EK: std::fmt::Display + Key> std::fmt::Display for Edge<EK> {{"
    )?;
    writeln!(
        edge,
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    )?;
    writeln!(
        edge,
        "        write!(f, \"{{}}({{}})\", self.get_type(), self.get_id())"
    )?;
    writeln!(edge, "    }}")?;
    writeln!(edge, "}}")?;

    new_files.add_content(edge_path.clone(), edge);

    Ok(())
}

/// Write ./edge_type.rs
pub(super) fn write_edge_type_rs<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let edge_path = schema_folder.join("edge_type.rs");
    let edges: Vec<_> = schema.edges().collect();
    let mut edge_type = String::new();

    writeln!(edge_type, "use super::*;")?;
    writeln!(edge_type, "use serde::{{Serialize, Deserialize}};")?;
    #[cfg(feature = "diff")]
    writeln!(edge_type, "use changesets::Changeset;")?;

    let attributes = vec![
        "Clone".to_string(),
        "Copy".to_string(),
        "Debug".to_string(),
        "PartialEq".to_string(),
        "Serialize".to_string(),
        "Deserialize".to_string(),
        #[cfg(feature = "diff")]
        "Changeset".to_string(),
    ];
    let attribute_s = attributes.join(", ");

    writeln!(edge_type, "")?;
    writeln!(edge_type, "#[derive({attribute_s})]")?;
    if !edges.is_empty() {
        writeln!(edge_type, "pub enum EdgeType {{")?;
        for n in &edges {
            let name = n.name.to_string();
            writeln!(edge_type, "    {name},")?;
        }
        writeln!(edge_type, "}}")?;
    } else {
        writeln!(edge_type, "pub struct EdgeType;")?;
    }

    writeln!(edge_type, "")?;
    writeln!(edge_type, "impl<EK> PartialEq<EdgeType> for Edge<EK> {{")?;
    writeln!(edge_type, "    fn eq(&self, _other: &EdgeType) -> bool {{")?;
    if !edges.is_empty() {
        writeln!(edge_type, "        match (_other, self) {{")?;
        for n in &edges {
            writeln!(
                edge_type,
                "           (EdgeType::{edge_type}, Edge::{edge_type}(_)) => true,",
                edge_type = n.name
            )?;
        }
        writeln!(edge_type, "           #[allow(unreachable_patterns)]")?;
        writeln!(edge_type, "           _ => false")?;
        writeln!(edge_type, "        }}")?;
    } else {
        writeln!(edge_type, "       true")?;
    }
    writeln!(edge_type, "    }}")?;
    writeln!(edge_type, "}}")?;

    writeln!(edge_type, "")?;
    writeln!(edge_type, "impl<EK> PartialEq<Edge<EK>> for EdgeType {{")?;
    writeln!(edge_type, "    fn eq(&self, other: &Edge<EK>) -> bool {{")?;
    writeln!(edge_type, "        other.eq(self)")?;
    writeln!(edge_type, "    }}")?;
    writeln!(edge_type, "}}")?;
    writeln!(edge_type, "")?;

    for e in &edges {
        let edge_name = &e.name;

        writeln!(
            edge_type,
            "impl<EK> PartialEq<EdgeType> for {edge_name}<EK> {{"
        )?;
        writeln!(edge_type, "    fn eq(&self, ty: &EdgeType) -> bool {{")?;
        writeln!(edge_type, "        matches!(ty, EdgeType::{edge_name})")?;
        writeln!(edge_type, "    }}")?;
        writeln!(edge_type, "}}")?;

        writeln!(edge_type, "")?;
        writeln!(
            edge_type,
            "impl<EK> PartialEq<{edge_name}<EK>> for EdgeType {{"
        )?;
        writeln!(
            edge_type,
            "    fn eq(&self, other: &{edge_name}<EK>) -> bool {{"
        )?;
        writeln!(edge_type, "        other.eq(self)")?;
        writeln!(edge_type, "    }}")?;
        writeln!(edge_type, "}}")?;
    }

    writeln!(edge_type, "")?;
    writeln!(edge_type, "impl std::fmt::Display for EdgeType {{")?;
    writeln!(
        edge_type,
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    )?;
    if !edges.is_empty() {
        writeln!(edge_type, "        match self {{")?;
        for e in &edges {
            let edge_name = &e.name;
            writeln!(
                edge_type,
                "            EdgeType::{edge_name} => write!(f, \"{edge_name}\"),"
            )?;
        }
        writeln!(edge_type, "        }}")?;
    } else {
        writeln!(edge_type, "write!(f, \"EdgeType\")")?;
    }
    writeln!(edge_type, "    }}")?;
    writeln!(edge_type, "}}")?;
    new_files.add_content(edge_path, edge_type);

    Ok(())
}

pub(super) fn write_edge_from<I>(
    e: &EdgeExp<I>,
    changeset: &ChangeSet<I>,
    parent_ty: &String,
) -> GenResult<String>
where
    I: Clone + PartialEq,
{
    let edge_type = &e.name;

    // Implement From Edge to Edge type
    let mut s = String::new();
    writeln!(s, "impl<EK> From<{parent_ty}<EK>> for {edge_type}<EK> {{")?;
    writeln!(s, "    fn from(other: {parent_ty}<EK>) -> Self {{")?;
    writeln!(s, "       {edge_type} {{")?;
    writeln!(s, "           id: other.id.into(),")?;
    for field_value in e.fields.iter() {
        let field_name = &field_value.name;
        let field_path = FieldPath::new_path(e.name.clone(), vec![field_name.clone()]);
        let changes = changeset.get_changes(field_path);
        let is_new = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedField(_)));

        if is_new {
            writeln!(s, "               {field_name}: Default::default()")?;
        } else {
            writeln!(s, "               {field_name}: other.{field_name},")?;
        }
    }
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    Ok(s)
}

pub(super) fn write_edge_endpoints<I: Debug + Ord>(
    schema: &Schema<I>,
    new_file: &mut GeneratedCode,
    nodes_path: &Path,
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

        writeln!(s, "")?;
        writeln!(s, "impl<NK> {node}<NK> {{")?;

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
                    edge_types.push(format!("EdgeType::{}", edge.name));
                }

                let edge_types_patterns = edge_types.join(" | ");

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
                    format!("Option<(&'a {output_edge_type}<EK>, &'a {end}<NK>)>")
                } else {
                    format!(
                        "impl Iterator<Item = (&'a {output_edge_type}<EK>, &'a {end}<NK>)> + 'a"
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
                writeln!(s, "   pub fn get_{node_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> SchemaResult<{return_type}, NK, EK, {schema_name}<NK, EK>>")?;
                writeln!(s, "   where")?;
                writeln!(s, "       NK: Key,")?;
                writeln!(s, "       EK: Key,")?;
                writeln!(s, "   {{")?;
                writeln!(s, "       #[allow(irrefutable_let_patterns)]")?;
                writeln!(s, "       Ok(g")?;
                writeln!(s, "           .get_{search_dir}_filter(self.get_id(), |e| matches!(e.get_type(), {edge_types_patterns}))?")?;
                writeln!(s, "           .filter_map(|e| Some((e.get_weight(), g.get_node_downcast(e.get_outer()).ok()?)))")?;
                // Cast the node into a specific type
                if let Some(_) = &only_edge_type {
                    writeln!(s, "           .map(|(e, n)| (Downcast::<_, _, &'a {output_edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e).unwrap(), n))")?;
                }
                if is_option {
                    writeln!(s, "           .next()")?;
                }
                writeln!(s, "       )")?;
                writeln!(s, "   }}")?;
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
                    writeln!(s, "   pub fn get_{edge_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> SchemaResult<impl Iterator<Item = (&'a {edge_type}<EK>, &'a Node<NK>)> + 'a, NK, EK, {schema_name}<NK, EK>>")?;
                    writeln!(s, "   where")?;
                    writeln!(s, "       NK: Key,")?;
                    writeln!(s, "       EK: Key,")?;
                    writeln!(s, "   {{")?;
                    writeln!(s, "       #[allow(irrefutable_let_patterns)]")?;
                    writeln!(s, "       Ok(g")?;
                    writeln!(s, "           .get_{search_dir}_filter(self.get_id(), |e| matches!(e.get_type(), EdgeType::{edge_type}))?")?;
                    writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node(e.get_outer()).unwrap()))")?;
                    writeln!(s, "       )")?;
                    writeln!(s, "   }}")?;

                // If there are no other edges we can safely cast the node to a specific type
                } else {
                    let is_option = endpoint.quantity.quantity == Bound::Included(1)
                        || endpoint.quantity.quantity == Bound::Excluded(2);
                    let return_type = if is_option {
                        format!("Option<(&'a {edge_type}<EK>, &'a {target_type}<NK>)>")
                    } else {
                        format!("impl Iterator<Item = (&'a {edge_type}<EK>, &'a {target_type}<NK>)> + 'a")
                    };

                    // Write get by edge type method
                    writeln!(s, "")?;
                    writeln!(s, "   pub fn get_{edge_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> SchemaResult<{return_type}, NK, EK, {schema_name}<NK, EK>>")?;
                    writeln!(s, "   where")?;
                    writeln!(s, "       NK: Key,")?;
                    writeln!(s, "       EK: Key,")?;
                    writeln!(s, "   {{")?;
                    writeln!(s, "       #[allow(irrefutable_let_patterns)]")?;
                    writeln!(s, "       Ok(g")?;
                    writeln!(s, "           .get_{search_dir}_filter(self.get_id(), |e| matches!(e.get_type(), EdgeType::{edge_type}))?")?;
                    writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node_downcast(e.get_outer()).unwrap()))")?;
                    if is_option {
                        writeln!(s, "           .next()")?;
                    }
                    writeln!(s, "       )")?;
                    writeln!(s, "   }}")?;
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
                    endpoint.source.to_string(),
                    endpoint.target.to_string(),
                    dir,
                )) {
                    continue;
                } else {
                    endpoint_impl.insert((
                        endpoint.source.to_string(),
                        endpoint.target.to_string(),
                        dir,
                    ));
                }

                // Check if the result can be
                let is_option = endpoint.quantity.quantity == Bound::Included(1)
                    || endpoint.quantity.quantity == Bound::Excluded(2);
                let return_type = if is_option {
                    format!("Option<(&'a {edge_type}<EK>, &'a {target_type}<NK>)>")
                } else {
                    format!(
                        "impl Iterator<Item = (&'a {edge_type}<EK>, &'a {target_type}<NK>)> + 'a"
                    )
                };

                // Write get by edge type method
                writeln!(s, "")?;
                writeln!(s, "   pub fn get_{edge_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> SchemaResult<{return_type}, NK, EK, {schema_name}<NK, EK>>")?;
                writeln!(s, "   where")?;
                writeln!(s, "       NK: Key,")?;
                writeln!(s, "       EK: Key,")?;
                writeln!(s, "   {{")?;
                writeln!(s, "       #[allow(irrefutable_let_patterns)]")?;
                writeln!(s, "       Ok(g")?;
                writeln!(s, "           .get_{search_dir}_filter(self.get_id(), |e| matches!(e.get_type(), EdgeType::{edge_type}))?")?;
                writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node(e.get_outer()).unwrap()))")?;
                writeln!(s, "           .filter_map(|(e, n)| Some((e, Downcast::<_, _, &'a {target_type}<NK>, {schema_name}<NK, EK>>::downcast(n).ok()?)))")?;
                if is_option {
                    writeln!(s, "           .next()")?;
                }
                writeln!(s, "       )")?;
                writeln!(s, "   }}")?;
            }
        }

        writeln!(s, "}}")?;

        let node_path = nodes_path.join(format!("{}.rs", node.to_snake_case()));
        new_file.add_content(node_path, s);
    }

    Ok(())
}
