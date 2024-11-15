use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{EdgeExp, EndPoint, LowerBound, NodeExp, Quantifier, Schema};
use build_script_shared::parsers::{Ident, ParserDeserialize};
use indexmap::IndexSet;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{format, Debug, Display, Write};
use std::path::Path;

use crate::common::{function_suffix, rename_attribute_name, search_dir, EdgeRepresentation};
use crate::{
    targets, CodeGenerator, Direction, GenError, GenResult, GeneratedCode, ToRustType, ToSnakeCase,
};

use super::{write_comments, write_fields, FieldFormatter};

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
        writeln!(s, "use indexmap::IndexMap;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use std::collections::HashSet;")?;
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
        write_comments(&mut s, &self.comments, Default::default())?;
        writeln!(s, "#[derive({attribute_s})]")?;
        writeln!(s, "pub struct {edge_name}<EK> {{")?;
        writeln!(s, "    pub(crate) id: EK,")?;
        write_fields(
            &mut s,
            &self.fields,
            FieldFormatter {
                indents: 1,
                include_visibility: true,
            },
        )?;
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "#[allow(unused)]")?;
        writeln!(s, "impl<EK> {edge_name}<EK> {{")?;
        writeln!(s, "    pub fn new(")?;
        write!(s, "        id: EK")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            writeln!(s, ",")?;
            let field_type = field_value.field_type.to_rust_type();
            write!(s, "        {field_name}: {field_type}")?;
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
            "impl<'b, NK, EK, S> Downcast<'b, NK, EK, &'b {edge_type}<EK>, S> for Edge<EK>"
        )?;
        writeln!(edge, "where")?;
        writeln!(edge, "    NK: Key,")?;
        writeln!(edge, "    EK: Key,")?;
        writeln!(edge, "    S: SchemaExt<NK, EK, E = Edge<EK>>")?;
        writeln!(edge, "{{")?;
        writeln!(
            edge,
            "    fn downcast<'a: 'b>(&'a self) -> SchemaResult<&'a {edge_type}<EK>, NK, EK, S> {{"
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
            "impl<'b, NK, EK, S> DowncastMut<'b, NK, EK, &'b mut {edge_type}<EK>, S> for Edge<EK>"
        )?;
        writeln!(edge, "where")?;
        writeln!(edge, "    NK: Key,")?;
        writeln!(edge, "    EK: Key,")?;
        writeln!(edge, "    S: SchemaExt<NK, EK, E = Edge<EK>>")?;
        writeln!(edge, "{{")?;
        writeln!(
            edge,
            "    fn downcast_mut<'a: 'b>(&'a mut self) -> SchemaResult<&'a mut {edge_type}<EK>, NK, EK, S> {{"
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
    writeln!(edge_type, "use typed_graph::GenericTypedError;")?;
    writeln!(edge_type, "use std::str::FromStr;")?;
    #[cfg(feature = "diff")]
    writeln!(edge_type, "use changesets::Changeset;")?;

    let attributes = vec![
        "Clone".to_string(),
        "Copy".to_string(),
        "Debug".to_string(),
        "PartialEq".to_string(),
        "Hash".to_string(),
        "Eq".to_string(),
        "PartialOrd".to_string(),
        "Ord".to_string(),
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

    writeln!(edge_type, "impl EdgeType {{")?;
    writeln!(edge_type, "    pub fn all() -> &'static [EdgeType] {{")?;
    writeln!(edge_type, "        &[")?;
    for n in &edges {
        let name = n.name.to_string();
        writeln!(edge_type, "            EdgeType::{name},")?;
    }
    writeln!(edge_type, "            ")?;
    writeln!(edge_type, "        ]")?;
    writeln!(edge_type, "    }}")?;
    writeln!(edge_type, "}}")?;

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

    writeln!(edge_type, "")?;
    writeln!(edge_type, "impl FromStr for EdgeType {{")?;
    writeln!(
        edge_type,
        "    type Err = GenericTypedError<String, String>;"
    )?;
    writeln!(edge_type, "")?;
    writeln!(
        edge_type,
        "    fn from_str(value: &str) -> Result<Self, Self::Err> {{"
    )?;
    writeln!(edge_type, "        match value {{")?;
    for e in &edges {
        let edge_name = &e.name;
        writeln!(
            edge_type,
            "            \"{edge_name}\" => Ok(EdgeType::{edge_name}),"
        )?;
    }
    writeln!(edge_type, "            _ => Err(GenericTypedError::UnrecognizedEdgeType(value.to_string(), EdgeType::all().into_iter().map(ToString::to_string).collect()))")?;
    writeln!(edge_type, "        }}")?;
    writeln!(edge_type, "    }}")?;
    writeln!(edge_type, "}}")?;

    new_files.add_content(edge_path, edge_type);

    Ok(())
}

/// Create a TryFrom implementation for the edge
/// In the case where no convertion can be found a manual one should be made instead
pub(super) fn write_edge_from<I>(
    e: &EdgeExp<I>,
    changeset: &ChangeSet<I>,
    parent_ty: &String,
) -> GenResult<String>
where
    I: Clone + PartialEq,
{
    let edge_type = &e.name;
    let mut omit_convertion = false;

    // Implement From Edge to Edge type
    let mut s = String::new();
    writeln!(
        s,
        "impl<EK> TryFrom<{parent_ty}<EK>> for {edge_type}<EK> {{"
    )?;
    writeln!(s, "    type Error = GenericTypedError<String, String>;")?;
    writeln!(s, "")?;
    writeln!(
        s,
        "    fn try_from(other: {parent_ty}<EK>) -> GenericTypedResult<Self, String, String> {{"
    )?;
    writeln!(s, "        Ok({edge_type} {{")?;
    writeln!(s, "            id: other.id.into(),")?;
    for field_value in e.fields.iter() {
        let field_name = &field_value.name;
        let field_path = FieldPath::new_path(e.name.clone(), vec![field_name.clone()]);
        let changes = changeset.get_changes(field_path.clone());
        let is_news = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedField(_)));
        if is_news {
            writeln!(s, "           {field_name}: Default::default(),")?;
        } else {
            let mut need_manual_implementation = false;
            let type_change = changes
                .iter()
                .filter_map(|c| {
                    if let SingleChange::EditedFieldType(v) = c {
                        Some(v)
                    } else {
                        None
                    }
                })
                .next();
            if let Some(type_change) = type_change {
                if !type_change
                    .old_type()
                    .is_gen_compatible(type_change.new_type())
                {
                    // We cannot trust the auto generated conversion so a manual one should be made instead
                    omit_convertion = true;
                    need_manual_implementation = true;
                }

                if need_manual_implementation {
                    writeln!(s, "           {field_name}: /* Insert convertion */,")?;
                } else {
                    writeln!(
                        s,
                        "           {field_name}: {},",
                        type_change.old_type().gen_convertion(format!("other.{field_name}"), true, type_change.new_type())
                    )?;
                }
            } else {
                writeln!(s, "           {field_name}: {},", field_value.field_type.gen_convertion(format!("other.{field_name}"), true, &field_value.field_type))?;
            }
        }
    }
    writeln!(s, "       }})")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    if omit_convertion {
        Ok(format!("/*Requires manual implementation\n{s}*/"))
    } else {
        Ok(s)
    }
}

pub(super) fn write_edge_endpoints<I: Debug + Ord>(
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

        writeln!(s, "")?;
        writeln!(s, "#[allow(unused)]")?;
        writeln!(s, "impl<NK> {node}<NK> {{")?;

        // Create getter functions for nodes and edges in a specific direction
        for (dir, endpoints) in directions {
            // Create maps storing how many types of edges go from and to each node type
            // It is important to use a vector as the same edgetype may be used multiple times
            // This allow the differnet functions to figure out if they can be cast to a specific type safely
            let mut grouped_by_end: BTreeMap<_, Vec<_>> = BTreeMap::new();
            let mut grouped_by_start: BTreeMap<_, Vec<_>> = BTreeMap::new();
            let mut grouped_by_edge: BTreeMap<_, IndexSet<_>> = BTreeMap::new();

            for (endpoint, edge) in &endpoints {
                let (start, end) = match dir {
                    Direction::Forward => (&endpoint.source, &endpoint.target),
                    Direction::Backwards => (&endpoint.target, &endpoint.source),
                };

                grouped_by_end.entry(end).or_default().push(edge);
                grouped_by_start.entry(start).or_default().push(&edge.name);
                grouped_by_edge
                    .entry(edge.name.clone())
                    .or_default()
                    .insert(end);
            }

            write_getter_with_node(&mut s, node, dir, &schema_name, &nodes, &grouped_by_end)?;

            write_getter_with_edge(
                &mut s,
                dir,
                &schema_name,
                &mut edge_impl,
                &endpoints,
                &grouped_by_edge,
            )?;

            write_getter_with_node_and_edge(
                &mut s,
                dir,
                &schema_name,
                &mut endpoint_impl,
                &endpoints,
            )?;
        }

        writeln!(s, "}}")?;

        let node_path = nodes_path.join(format!("{}.rs", node.to_snake_case()));
        new_file.add_content(node_path, s);
    }

    Ok(())
}

/// Create getter functions with a fixed node type
fn write_getter_with_node<I: Debug + Ord>(
    s: &mut String,
    node: &Ident<I>,
    dir: Direction,
    schema_name: &str,
    nodes: &BTreeMap<&Ident<I>, &NodeExp<I>>,
    grouped_by_end: &BTreeMap<&Ident<I>, Vec<&&EdgeExp<I>>>,
) -> GenResult<()> {
    for (end, edges) in grouped_by_end {
        let mut edge_types = Vec::new();
        let mut edge_names = Vec::new();
        for edge in edges {
            edge_types.push(format!("EdgeType::{}", edge.name));
            edge_names.push(edge.name.to_string());
        }

        let edge_name_list = edge_names.join(", ");
        let edge_types_patterns = edge_types.join(" | ");

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
                    Direction::Forward => node == &endpoint.source && end == &&endpoint.target,
                    Direction::Backwards => node == &endpoint.target && end == &&endpoint.source,
                })
                // Limit based on
                .map(|(_, endpoint)| match dir {
                    Direction::Forward => &endpoint.outgoing_quantity,
                    Direction::Backwards => &endpoint.incoming_quantity,
                })
                .fold(edge_repr, |repr, quantity| {
                    EdgeRepresentation::from_quantity(quantity).max(repr)
                });
        }

        let output_edge_type =
            only_edge_type.map_or_else(|| "Edge".to_string(), |edge| edge.name.to_string());

        let return_type = edge_repr.get_return_type_rust(&output_edge_type, format!("&'a {end}<NK>"), schema_name);

        let rename_attribute = nodes.get(node).and_then(|n| {
            n.attributes
                .get_functions(rename_attribute_name(dir))
                .into_iter()
                .filter_map(|attr| attr.values.get(0).zip(attr.values.get(1)))
                .find(|(_, rename_node)| rename_node == end)
        });

        let node_func_name = if let Some((new_name, _)) = rename_attribute {
            new_name.to_snake_case()
        } else {
            let function_name = end.to_snake_case();
            format!("get_{function_name}_{}", function_suffix(dir))
        };

        // Write get by node type method
        writeln!(s, "")?;
        writeln!(s, "   pub fn {node_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> {return_type}")?;
        writeln!(s, "   where")?;
        writeln!(s, "       NK: Key,")?;
        writeln!(s, "       EK: Key,")?;
        writeln!(s, "   {{")?;
        writeln!(s, "       Ok(g")?;
        writeln!(s, "           .get_{}_filter(self.get_id(), |e| matches!(e.get_type(), {edge_types_patterns}))?", search_dir(dir))?;
        writeln!(s, "           .filter_map(|e| Some((e.get_weight(), g.get_node_downcast(e.get_outer()).ok()?)))")?;
        // Cast the node into a specific type
        if only_edge_type.is_some() {
            writeln!(s, "           .map(|(e, n)| (Downcast::<_, _, &'a {output_edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e).unwrap(), n))")?;
        }
        edge_repr.collect_results_rust(edge_name_list, s)?;
        writeln!(s, "       )")?;
        writeln!(s, "   }}")?;
    }

    Ok(())
}

/// Create getter functions with a fixed edge type
fn write_getter_with_edge<I: Debug + Ord>(
    s: &mut String,
    dir: Direction,
    schema_name: &str,
    edge_impl: &mut HashSet<(String, Direction)>,
    endpoints: &Vec<(&EndPoint<I>, &EdgeExp<I>)>,
    grouped_by_edge: &BTreeMap<String, IndexSet<&Ident<I>>>,
) -> GenResult<()> {
    for (_, edge) in endpoints {
        let edge_type = &edge.name;

        let edge_func_name =
            if let Some(new_name) = edge.attributes.get_key_value(rename_attribute_name(dir)) {
                new_name.value.to_snake_case()
            } else {
                let function_name = edge.name.to_snake_case();
                format!("get_{function_name}_{}", function_suffix(dir))
            };

        let target_types = grouped_by_edge.get(edge.name.as_str()).unwrap();
        let target_vec = target_types.into_iter().collect::<Vec<_>>();
        let (target_type, requires_downcast) = match target_vec.as_slice() {
            [] => Err(GenError::ExportFailed(format!(
                "Failed to find {} edge for {edge_type}",
                function_suffix(dir)
            ))),

            // Use specific type
            [n] => Ok((format!("&'a {n}<NK>"), true)),

            // Use any of the specific types
            nodes if nodes.len() < 10 => {
                let len = nodes.len();
                let generics = nodes
                    .iter()
                    .map(|n| format!("&'a {n}<NK>"))
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok((format!("Either{len}<{generics}>"), true))
            }

            // Fallback if no specific type can be found
            _ => Ok(("&'a Node<NK>".to_string(), false)),
        }?;

        // If there are multiple of the same edge type to a node there should only be one function implementation
        if edge_impl.contains(&(edge.name.clone(), dir)) {
            continue;
        } else {
            edge_impl.insert((edge.name.clone(), dir));
        }

        let mut edge_repr = EdgeRepresentation::Result;
        for (endpoint, e) in endpoints {
            if e != edge {
                continue;
            }

            let quantity = match dir {
                Direction::Forward => &endpoint.outgoing_quantity,
                Direction::Backwards => &endpoint.incoming_quantity,
            };

            edge_repr = EdgeRepresentation::from_quantity(quantity).max(edge_repr);
        }

        let return_type = edge_repr.get_return_type_rust(edge_type, target_type, schema_name);

        // Write get by edge type method
        writeln!(s, "")?;
        writeln!(s, "   pub fn {edge_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> {return_type}")?;
        writeln!(s, "   where")?;
        writeln!(s, "       NK: Key,")?;
        writeln!(s, "       EK: Key,")?;
        writeln!(s, "   {{")?;
        writeln!(s, "       Ok(g")?;
        writeln!(s, "           .get_{}_filter(self.get_id(), |e| matches!(e.get_type(), EdgeType::{edge_type}))?", search_dir(dir))?;
        if requires_downcast {
            writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node_downcast(e.get_outer()).unwrap()))")?;
        } else {
            writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node(e.get_outer()).unwrap()))")?;
        }
        edge_repr.collect_results_rust(edge_type, s)?;
        writeln!(s, "       )")?;
        writeln!(s, "   }}")?;
    }
    Ok(())
}

/// Create getter functions with a fixed node and edge type
fn write_getter_with_node_and_edge<I: Ord + Debug>(
    s: &mut String,
    dir: Direction,
    schema_name: &str,
    endpoint_impl: &mut HashSet<(String, String, Direction)>,
    endpoints: &Vec<(&EndPoint<I>, &EdgeExp<I>)>,
) -> GenResult<()> {
    for (endpoint, edge) in endpoints {
        let edge_type = &edge.name;

        let target_type = match dir {
            Direction::Forward => &endpoint.target,
            Direction::Backwards => &endpoint.source,
        };

        let edge_func_name = if let Some(new_name) = endpoint
            .attributes
            .get_key_value(rename_attribute_name(dir))
        {
            new_name.value.to_snake_case()
        } else {
            let edge_name = edge.name.to_snake_case();
            let target_name = target_type.to_snake_case();
            format!("get_{target_name}_via_{edge_name}_{}", function_suffix(dir))
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

        let quantity = match dir {
            Direction::Forward => &endpoint.outgoing_quantity,
            Direction::Backwards => &endpoint.incoming_quantity,
        };
        let edge_repr = EdgeRepresentation::from_quantity(quantity);
        let return_type = edge_repr.get_return_type_rust(edge_type, format!("&'a {target_type}<NK>"), schema_name);

        // Write get by edge type method
        writeln!(s, "")?;
        writeln!(s, "   pub fn {edge_func_name}<'a, EK>(&'a self, g: &'a TypedGraph<NK, EK, {schema_name}<NK, EK>>) -> {return_type}")?;
        writeln!(s, "   where")?;
        writeln!(s, "       NK: Key,")?;
        writeln!(s, "       EK: Key,")?;
        writeln!(s, "   {{")?;
        writeln!(s, "       Ok(g")?;
        writeln!(s, "           .get_{}_filter(self.get_id(), |e| matches!(e.get_type(), EdgeType::{edge_type}))?", search_dir(dir))?;
        writeln!(s, "           .map(|e| (Downcast::<_, _, &'a {edge_type}<EK>, {schema_name}<NK, EK>>::downcast(e.get_weight()).unwrap(), g.get_node_downcast(e.get_outer()).unwrap()))")?;
        edge_repr.collect_results_rust(edge_type, s)?;
        writeln!(s, "       )")?;
        writeln!(s, "   }}")?;
    }

    Ok(())
}