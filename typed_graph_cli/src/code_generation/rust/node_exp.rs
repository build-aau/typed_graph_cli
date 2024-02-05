use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{NodeExp, Schema, Visibility};
use build_script_shared::InputType;
use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

impl<I> CodeGenerator<targets::Rust> for NodeExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let node_type = &self.name;
        let node_path = p.as_ref().join(format!(
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
        write!(s, "pub struct {node_type}<NK>")?;
        writeln!(s, "   {{")?;
        write!(s, "    pub(crate) id: NK")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;

            writeln!(s, ",")?;
            for comment in field_value.comments.iter_doc() {
                writeln!(s, "   /// {comment}")?;
            }
            let vis = match field_value.visibility {
                Visibility::Local => "pub(crate) ",
                Visibility::Public => "pub ",
            };
            let field_type = &field_value.field_type.to_rust_type();
            write!(s, "    {vis}{field_name}: {field_type}")?;
        }
        writeln!(s, "")?;
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "impl<NK> {node_type}<NK> {{")?;
        writeln!(s, "    pub fn new(")?;
        write!(s, "       id: NK")?;
        for field_value in self.fields.iter() {
            let field_type = &field_value.field_type.to_rust_type();
            let field_name = &field_value.name;
            writeln!(s, ",")?;
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
        writeln!(s, "impl<NK> Typed for {node_type}<NK> {{")?;
        writeln!(s, "    type Type = NodeType;")?;
        writeln!(s, "    fn get_type(&self) -> NodeType {{")?;
        writeln!(s, "       NodeType::{node_type}")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "impl<NK: Key> Id<NK> for {node_type}<NK> {{")?;
        writeln!(s, "    fn get_id(&self) -> NK {{")?;
        writeln!(s, "        self.id")?;
        writeln!(s, "    }}")?;

        writeln!(s, "")?;
        writeln!(s, "    fn set_id(&mut self, id: NK) {{")?;
        writeln!(s, "        self.id = id")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        let name_type = self
            .fields
            .get_field("name")
            .map(|field_value| field_value.field_type.to_string());

        if let Some(name_type) = name_type {
            writeln!(s, "")?;
            writeln!(s, "impl<NK> Name for {node_type}<NK> {{")?;
            writeln!(s, "    type Name = {name_type};")?;
            writeln!(s, "    fn get_name(&self) -> Option<&Self::Name> {{")?;
            writeln!(s, "       Some(&self.name)")?;
            writeln!(s, "    }}")?;
            writeln!(s, "}}")?;
        }

        writeln!(s, "")?;
        writeln!(
            s,
            "impl<EK: std::fmt::Display + Key> std::fmt::Display for {node_type}<EK> {{"
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
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}

/// Write ./nodes.rs
pub(super) fn write_nodes_rs<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let node_path = schema_folder.join("node.rs");

    let nodes: Vec<_> = schema.nodes().collect();
    let mut node = String::new();

    writeln!(node, "#[allow(unused_imports)]")?;
    writeln!(node, "use super::super::imports::*;")?;
    writeln!(node, "use super::*;")?;
    writeln!(node, "use std::fmt::Debug;")?;
    writeln!(node, "use typed_graph::*;")?;
    writeln!(node, "use serde::{{Serialize, Deserialize}};")?;
    #[cfg(feature = "diff")]
    writeln!(node, "use changesets::Changeset;")?;

    let attributes = vec![
        "Clone".to_string(),
        "Debug".to_string(),
        "Serialize".to_string(),
        "Deserialize".to_string(),
        #[cfg(feature = "diff")]
        "Changeset".to_string(),
    ];
    let attribute_s = attributes.join(", ");

    writeln!(node, "")?;
    writeln!(node, "#[derive({attribute_s})]")?;
    if nodes.is_empty() {
        writeln!(node, "pub struct Node<NK> {{")?;
        writeln!(node, "    id: NK")?;
        writeln!(node, "}}")?;
    } else {
        writeln!(node, "pub enum Node<NK> {{")?;
        for n in &nodes {
            let node_type = &n.name;
            writeln!(node, "    {node_type}({node_type}<NK>),")?;
        }
        writeln!(node, "}}")?;
    }

    writeln!(node, "")?;
    writeln!(node, "impl<NK: Key> NodeExt<NK> for Node<NK>{{}}")?;

    writeln!(node, "")?;
    writeln!(node, "impl<NK> Typed for Node<NK> {{")?;
    writeln!(node, "    type Type = NodeType;")?;
    writeln!(node, "    fn get_type(&self) -> NodeType {{")?;
    if !nodes.is_empty() {
        writeln!(node, "        match self {{")?;
        for n in &nodes {
            let node_type = &n.name;
            writeln!(
                node,
                "            Node::{node_type}(_) => NodeType::{node_type},"
            )?;
        }
        writeln!(node, "        }}")?;
    } else {
        writeln!(node, "        NodeType")?;
    }
    writeln!(node, "    }}")?;
    writeln!(node, "}}")?;

    writeln!(node, "")?;
    writeln!(node, "impl<NK: Key> Id<NK> for Node<NK> {{")?;
    writeln!(node, "    fn get_id(&self) -> NK {{")?;
    if !nodes.is_empty() {
        writeln!(node, "        match self {{")?;
        for n in &nodes {
            let node_type = &n.name;
            writeln!(node, "            Node::{node_type}(e) => e.get_id(),")?;
        }
        writeln!(node, "        }}")?;
    } else {
        writeln!(node, "        self.id")?;
    }
    writeln!(node, "    }}")?;

    writeln!(node, "")?;
    writeln!(node, "    fn set_id(&mut self, id: NK) {{")?;
    if !nodes.is_empty() {
        writeln!(node, "        match self {{")?;
        for n in &nodes {
            let node_type = &n.name;
            writeln!(node, "            Node::{node_type}(e) => e.set_id(id),")?;
        }
        writeln!(node, "        }}")?;
    } else {
        writeln!(node, "        self.id = id;")?;
    }
    writeln!(node, "    }}")?;
    writeln!(node, "}}")?;

    // Check if there is only a single type used for names
    let name_type = nodes.iter().map(|e| e.fields.get_field("name")).fold(
        Some(HashSet::new()),
        |acc, name_field| {
            acc.zip(name_field)
                .map(|(mut field_types, field_value)| {
                    field_types.insert(field_value.field_type.to_string());
                    field_types
                })
        },
    );

    if let Some(name_type) = name_type {
        if name_type.len() == 1 {
            let name_type = name_type.into_iter().next().unwrap();
            writeln!(node, "")?;
            writeln!(node, "impl<NK> Name for Node<NK> {{")?;
            writeln!(node, "    type Name = {name_type};")?;
            writeln!(node, "    fn get_name(&self) -> Option<&Self::Name> {{")?;
            writeln!(node, "       match self {{")?;
            for n in &nodes {
                let node_type = &n.name;

                if n.fields.has_field("name") {
                    writeln!(node, "        Node::{node_type}(e) => e.get_name(),")?;
                } else {
                    writeln!(node, "        Node::{node_type}(e) => None,")?;
                }
            }
            writeln!(node, "       }}")?;
            writeln!(node, "    }}")?;
            writeln!(node, "}}")?;
        }
    }

    for n in &nodes {
        let node_type = &n.name;

        writeln!(node, "")?;
        writeln!(node, "impl<NK> From<{node_type}<NK>> for Node<NK> {{")?;
        writeln!(node, "    fn from(other: {node_type}<NK>) -> Self {{")?;
        writeln!(node, "        Self::{node_type}(other)")?;
        writeln!(node, "    }}")?;
        writeln!(node, "}}")?;
    }

    for n in &nodes {
        let node_type = &n.name;

        writeln!(node, "")?;
        writeln!(
            node,
            "impl<'a, NK, EK, S> Downcast<'a, NK, EK, &'a {node_type}<NK>, S> for Node<NK>"
        )?;
        writeln!(node, "where")?;
        writeln!(node, "    NK: Key,")?;
        writeln!(node, "    EK: Key,")?;
        writeln!(node, "    S: SchemaExt<NK, EK, N = Node<NK>>")?;
        writeln!(node, "{{")?;
        writeln!(
            node,
            "    fn downcast(&'a self) -> SchemaResult<&'a {node_type}<NK>, NK, EK, S> {{"
        )?;
        writeln!(node, "        match self {{")?;
        writeln!(node, "            Node::{node_type}(n) => Ok(n),")?;
        writeln!(node, "            #[allow(unreachable_patterns)]")?;
        writeln!(node, "            n => Err(TypedError::DownCastFailed(\"{node_type}\".to_string(), n.get_type().to_string()))")?;
        writeln!(node, "        }}")?;
        writeln!(node, "    }}")?;
        writeln!(node, "}}")?;
    }

    for n in &nodes {
        let node_type = &n.name;

        writeln!(node, "")?;
        writeln!(
            node,
            "impl<'a, NK, EK, S> DowncastMut<'a, NK, EK, &'a mut {node_type}<NK>, S> for Node<NK>"
        )?;
        writeln!(node, "where")?;
        writeln!(node, "    NK: Key,")?;
        writeln!(node, "    EK: Key,")?;
        writeln!(node, "    S: SchemaExt<NK, EK, N = Node<NK>>")?;
        writeln!(node, "{{")?;
        writeln!(
            node,
            "    fn downcast_mut(&'a mut self) -> SchemaResult<&'a mut {node_type}<NK>, NK, EK, S> {{"
        )?;
        writeln!(node, "        match self {{")?;
        writeln!(node, "            Node::{node_type}(n) => Ok(n),")?;
        writeln!(node, "            #[allow(unreachable_patterns)]")?;
        writeln!(node, "            n => Err(TypedError::DownCastFailed(\"{node_type}\".to_string(), n.get_type().to_string()))")?;
        writeln!(node, "        }}")?;
        writeln!(node, "    }}")?;
        writeln!(node, "}}")?;
    }

    writeln!(node, "")?;
    writeln!(
        node,
        "impl<NK: std::fmt::Display + Key> std::fmt::Display for Node<NK> {{"
    )?;
    writeln!(
        node,
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    )?;
    writeln!(
        node,
        "        write!(f, \"{{}}({{}})\", self.get_type(), self.get_id())"
    )?;
    writeln!(node, "    }}")?;
    writeln!(node, "}}")?;

    new_files.add_content(node_path, node);

    Ok(())
}

/// Write ./node_type.rs
pub(super) fn write_node_type_rs<I: Ord>(
    schema: &Schema<I>,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let node_path = schema_folder.join("node_type.rs");
    let nodes: Vec<_> = schema.nodes().collect();

    let mut node_type = String::new();
    writeln!(node_type, "use super::*;")?;
    writeln!(node_type, "use serde::{{Serialize, Deserialize}};")?;
    #[cfg(feature = "diff")]
    writeln!(node_type, "use changesets::Changeset;")?;

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

    writeln!(node_type, "")?;
    writeln!(node_type, "#[derive({attribute_s})]")?;
    if !nodes.is_empty() {
        writeln!(node_type, "pub enum NodeType {{")?;
        for n in &nodes {
            let name = n.name.to_string();
            writeln!(node_type, "    {name},")?;
        }
        writeln!(node_type, "}}")?;
    } else {
        writeln!(node_type, "pub struct NodeType;")?;
    }

    writeln!(node_type, "")?;
    writeln!(node_type, "impl<NK> PartialEq<NodeType> for Node<NK> {{")?;
    writeln!(node_type, "    fn eq(&self, _other: &NodeType) -> bool {{")?;
    if !nodes.is_empty() {
        writeln!(node_type, "        match (_other, self) {{")?;
        for n in &nodes {
            let node_name = &n.name;
            writeln!(
                node_type,
                "           (NodeType::{node_name}, Node::{node_name}(_)) => true,"
            )?;
        }
        writeln!(node_type, "           _ => false,")?;
        writeln!(node_type, "        }}")?;
    } else {
        writeln!(node_type, "        true")?;
    }
    writeln!(node_type, "    }}")?;
    writeln!(node_type, "}}")?;

    writeln!(node_type, "")?;
    writeln!(node_type, "impl<NK> PartialEq<Node<NK>> for NodeType {{")?;
    writeln!(node_type, "    fn eq(&self, other: &Node<NK>) -> bool {{")?;
    writeln!(node_type, "       other.eq(self)")?;
    writeln!(node_type, "    }}")?;
    writeln!(node_type, "}}")?;

    writeln!(node_type, "")?;
    for n in &nodes {
        let node_name = &n.name;

        writeln!(
            node_type,
            "impl<NK> PartialEq<NodeType> for {node_name}<NK> {{"
        )?;
        writeln!(node_type, "    fn eq(&self, ty: &NodeType) -> bool {{")?;
        writeln!(node_type, "        matches!(ty, NodeType::{node_name})")?;
        writeln!(node_type, "    }}")?;
        writeln!(node_type, "}}")?;

        writeln!(node_type, "")?;
        writeln!(
            node_type,
            "impl<EK> PartialEq<{node_name}<EK>> for NodeType {{"
        )?;
        writeln!(
            node_type,
            "    fn eq(&self, other: &{node_name}<EK>) -> bool {{"
        )?;
        writeln!(node_type, "        other.eq(self)")?;
        writeln!(node_type, "    }}")?;
        writeln!(node_type, "}}")?;
    }

    writeln!(node_type, "")?;
    writeln!(node_type, "impl std::fmt::Display for NodeType {{")?;
    writeln!(
        node_type,
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    )?;
    if !nodes.is_empty() {
        writeln!(node_type, "        match self {{")?;
        for n in &nodes {
            let node_name = &n.name;
            writeln!(
                node_type,
                "            NodeType::{node_name} => write!(f, \"{node_name}\"),"
            )?;
        }
        writeln!(node_type, "        }}")?;
    } else {
        writeln!(node_type, "        write!(f, \"NodeType\")")?;
    }
    writeln!(node_type, "    }}")?;
    writeln!(node_type, "}}")?;

    new_files.add_content(node_path, node_type);

    Ok(())
}

pub(super) fn write_node_from<I: Clone + PartialEq>(
    n: &NodeExp<I>,
    changeset: &ChangeSet<I>,
    parent_ty: &String,
) -> GenResult<String> {
    let node_type = &n.name;

    let mut s = String::new();
    writeln!(s, "")?;
    writeln!(s, "impl<NK> From<{parent_ty}<NK>> for {node_type}<NK> {{")?;
    writeln!(s, "    fn from(other: {parent_ty}<NK>) -> Self {{")?;
    writeln!(s, "       {node_type} {{")?;
    writeln!(s, "           id: other.id.into(),")?;
    for field_value in n.fields.iter() {
        let field_name = &field_value.name;
        let field_path = FieldPath::new_path(n.name.clone(), vec![field_name.clone()]);
        let changes = changeset.get_changes(field_path);
        let is_removed = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedField(_)));
        if is_removed {
            writeln!(s, "           {field_name}: Default::default()")?;
        } else {
            writeln!(s, "           {field_name}: other.{field_name}.into(),")?;
        }
    }
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    Ok(s)
}
