use super::{
    write_edge_endpoints, write_edge_type_rs, write_edges_rs, write_node_type_rs, write_nodes_rs,
};
use crate::*;
use build_script_lang::schema::{Schema, SchemaStm};
use build_script_shared::parsers::Ident;
use std::collections::HashSet;
use std::fmt::{Debug, Write};
use std::fs::create_dir;
use std::path::Path;

impl<I> CodeGenerator<targets::Rust> for (&Project, &Schema<I>)
where
    I: Ord + Debug,
{
    fn get_filename(&self) -> String {
        self.1.version.to_string().replace(".", "_").to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let schema_folder = p
            .as_ref()
            .join(CodeGenerator::<targets::Rust>::get_filename(self));
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
            &self.1,
            &mut new_files,
            &nodes_folder,
            &structs_folder,
            &edges_folder,
            &types_folder,
        )?;
        write_nodes_rs(&self.1, &mut new_files, &schema_folder)?;
        write_node_type_rs(&self.1, &mut new_files, &schema_folder)?;
        write_edges_rs(&self.1, &mut new_files, &schema_folder)?;
        write_edge_type_rs(&self.1, &mut new_files, &schema_folder)?;
        write_mod(
            &self.1,
            &mut new_files,
            &schema_folder,
            &nodes_folder,
            &structs_folder,
            &edges_folder,
            &types_folder,
        )?;
        write_schema_impl_rs(&self.1, self.0, &mut new_files, &schema_folder)?;
        write_edge_endpoints(&self.1, &mut new_files, &nodes_folder)?;

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
                CodeGenerator::<targets::Rust>::aggregate_content(n, &nodes_folder)
            }
            SchemaStm::Struct(n) => {
                CodeGenerator::<targets::Rust>::aggregate_content(n, &structs_folder)
            }
            SchemaStm::Edge(n) => {
                CodeGenerator::<targets::Rust>::aggregate_content(n, &edges_folder)
            }
            SchemaStm::Enum(n) => {
                CodeGenerator::<targets::Rust>::aggregate_content(n, &types_folder)
            }
            SchemaStm::Import(_) => Ok(GeneratedCode::new()),
        }?;

        new_files.append(added_files);
    }

    Ok(())
}

fn write_mod<I>(
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
    // Write ./{nodes|edges|types}/mod.rs
    let nodes_mod_path = nodes_folder.join("mod.rs");
    let structs_mod_path = structs_folder.join("mod.rs");
    let edges_mod_path = edges_folder.join("mod.rs");
    let types_mod_path = types_folder.join("mod.rs");

    let mut nodes_mod = String::new();
    let mut structs_mod = String::new();
    let mut edges_mod = String::new();
    let mut types_mod = String::new();

    for stm in schema.iter() {
        let (filename, f) = match stm {
            SchemaStm::Node(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &mut nodes_mod,
            ),
            SchemaStm::Struct(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &mut structs_mod,
            ),
            SchemaStm::Edge(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &mut edges_mod,
            ),
            SchemaStm::Enum(n) => (
                CodeGenerator::<targets::Rust>::get_filename(n),
                &mut types_mod,
            ),
            SchemaStm::Import(_) => continue,
        };

        writeln!(f, "mod {};", filename)?;
        writeln!(f, "#[allow(unused)]")?;
        writeln!(f, "pub use {}::*;", filename)?;
    }

    new_files.add_content(nodes_mod_path, nodes_mod);
    new_files.add_content(structs_mod_path, structs_mod);
    new_files.add_content(edges_mod_path, edges_mod);
    new_files.add_content(types_mod_path, types_mod);

    // Write ./mod.rs
    let schema_mod_path = schema_folder.join("mod.rs");
    let mut schema_mod = String::new();
    writeln!(schema_mod, "mod node;")?;
    writeln!(schema_mod, "pub mod structs;")?;
    writeln!(schema_mod, "mod edge;")?;
    writeln!(schema_mod, "pub mod types;")?;
    writeln!(schema_mod, "pub mod nodes;")?;
    writeln!(schema_mod, "pub mod edges;")?;
    writeln!(schema_mod, "mod edge_type;")?;
    writeln!(schema_mod, "mod node_type;")?;
    writeln!(schema_mod, "mod schema;")?;
    writeln!(schema_mod, "mod imports;")?;
    writeln!(schema_mod, "")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use schema::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use edge_type::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use node_type::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use node::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use structs::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use edge::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use nodes::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use edges::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use types::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use imports::*;")?;
    writeln!(schema_mod, "#[allow(unused)]")?;
    writeln!(schema_mod, "pub use super::imports::*;")?;

    let imports_path = schema_folder.join("imports.rs");
    new_files.create_file(imports_path);

    new_files.add_content(schema_mod_path, schema_mod);

    Ok(())
}

fn write_schema_impl_rs<I: Ord>(
    schema: &Schema<I>,
    project: &Project,
    new_files: &mut GeneratedCode,
    schema_folder: &Path,
) -> GenResult<()> {
    let schema_path = schema_folder.join("schema.rs");
    let schema_name = schema.version.replace(".", "_");
    let schema_version = &schema.version;

    let mut schema_rs = String::new();
    writeln!(schema_rs, "use super::*;")?;
    writeln!(schema_rs, "use std::fmt::Debug;")?;
    writeln!(schema_rs, "use std::marker::PhantomData;")?;
    writeln!(schema_rs, "use typed_graph::*;")?;
    writeln!(schema_rs, "use serde::{{Serialize, Deserialize}};")?;
    writeln!(schema_rs, "use typed_graph::GenericTypedError;")?;
    writeln!(schema_rs, "use std::str::FromStr;")?;
    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "#[allow(unused)]")?;
    writeln!(
        schema_rs,
        "pub type {schema_name}Graph<NK, EK> = TypedGraph<NK, EK, {schema_name}<NK, EK>>;"
    )?;
    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "pub const {schema_name}_NAME: &'static str = \"{schema_version}\";")?;
    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]")?;

    writeln!(schema_rs, "#[serde(bound = \"NK: Clone, EK: Clone\")]")?;
    writeln!(schema_rs, "#[serde(try_from = \"String\")]")?;
    writeln!(schema_rs, "#[serde(into = \"String\")]")?;
    writeln!(schema_rs, "pub struct {schema_name}<NK, EK> {{")?;
    writeln!(schema_rs, "    ek: PhantomData<EK>,")?;
    writeln!(schema_rs, "    nk: PhantomData<NK>,")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> {schema_name}<NK, EK> {{"
    )?;
    writeln!(schema_rs, "    pub const fn name() -> &'static str {{")?;
    writeln!(schema_rs, "        \"{schema_version}\"")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "")?;
    
    let parents = project.get_parents(&schema_version);
    writeln!(schema_rs, "    pub const fn parents() -> &'static [&'static str] {{")?;
    writeln!(schema_rs, "        &[")?;
    for parent in parents.iter() {
        writeln!(schema_rs, "            \"{parent}\",")?;
    }
    writeln!(schema_rs, "        ]")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> SchemaExt<NK, EK> for {schema_name}<NK, EK>"
    )?;
    writeln!(schema_rs, "where")?;
    writeln!(schema_rs, "   NK: Key,")?;
    writeln!(schema_rs, "   EK: Key")?;
    writeln!(schema_rs, "{{")?;
    writeln!(schema_rs, "   type N = Node<NK>;")?;
    writeln!(schema_rs, "   type E = Edge<EK>;")?;

    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "    fn name(&self) -> String {{")?;
    writeln!(schema_rs, "        self.to_string()")?;
    writeln!(schema_rs, "    }}")?;

    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "    fn allow_node(&self, _node_ty: NodeType) -> Result<(), DisAllowedNode> {{"
    )?;
    writeln!(schema_rs, "        Ok(())")?;
    writeln!(schema_rs, "    }}")?;

    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "    #[allow(unused_variables)]")?;
    writeln!(schema_rs, "    fn allow_edge(&self, outgoing_edge_count: usize, incoming_edge_count: usize, edge_ty: EdgeType, source_ty: NodeType, target_ty: NodeType) -> Result<(), DisAllowedEdge> {{")?;
    writeln!(
        schema_rs,
        "        match (edge_ty, source_ty, target_ty) {{"
    )?;
    for e in schema.edges() {
        let edge_type = &e.name;
        for ((source, target), endpoint) in &e.endpoints {
            if let Some((_, upper)) = endpoint.incoming_quantity.bounds {
                writeln!(schema_rs, "            (EdgeType::{edge_type}, NodeType::{source}, NodeType::{target}) if incoming_edge_count > {upper} => Err(DisAllowedEdge::ToManyIncoming),")?;
            }

            if let Some((_, upper)) = endpoint.outgoing_quantity.bounds {
                writeln!(schema_rs, "            (EdgeType::{edge_type}, NodeType::{source}, NodeType::{target}) if outgoing_edge_count > {upper} => Err(DisAllowedEdge::ToManyOutgoing),")?;
            }

            writeln!(schema_rs, "            (EdgeType::{edge_type}, NodeType::{source}, NodeType::{target}) => Ok(()),")?;
        }
    }
    writeln!(schema_rs, "            #[allow(unreachable_patterns)]")?;
    writeln!(
        schema_rs,
        "            _ => Err(DisAllowedEdge::InvalidType)"
    )?;
    writeln!(schema_rs, "        }}")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> Default for {schema_name}<NK, EK> {{"
    )?;
    writeln!(schema_rs, "    fn default() -> Self {{")?;
    writeln!(schema_rs, "        {schema_name} {{")?;
    writeln!(schema_rs, "            ek: PhantomData,")?;
    writeln!(schema_rs, "            nk: PhantomData")?;
    writeln!(schema_rs, "        }}")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> TryFrom<String> for {schema_name}<NK, EK> {{"
    )?;
    writeln!(
        schema_rs,
        "    type Error = GenericTypedError<String, String>;"
    )?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "    fn try_from(value: String) -> Result<Self, Self::Error> {{"
    )?;
    writeln!(schema_rs, "        value.parse()")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> FromStr for {schema_name}<NK, EK> {{"
    )?;
    writeln!(
        schema_rs,
        "    type Err = GenericTypedError<String, String>;"
    )?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "    fn from_str(value: &str) -> Result<Self, Self::Err> {{"
    )?;
    writeln!(schema_rs, "        if value != \"{schema_version}\" {{")?;
    writeln!(
        schema_rs,
        "            return Err(GenericTypedError::InvalidSchemaName(value.to_string(), \"{schema_version}\".to_string()));"
    )?;
    writeln!(schema_rs, "        }}")?;
    writeln!(schema_rs, "        Ok({schema_name}::default())")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(
        schema_rs,
        "impl<NK, EK> Into<String> for {schema_name}<NK, EK> {{"
    )?;
    writeln!(schema_rs, "    fn into(self) -> String {{")?;
    writeln!(schema_rs, "        self.to_string()")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;
    writeln!(schema_rs, "")?;
    writeln!(schema_rs, "impl<NK, EK> ToString for {schema_name}<NK, EK> {{")?;
    writeln!(schema_rs, "    fn to_string(&self) -> String {{")?;
    writeln!(schema_rs, "        \"{schema_version}\".to_string()")?;
    writeln!(schema_rs, "    }}")?;
    writeln!(schema_rs, "}}")?;

    new_files.add_content(schema_path, schema_rs);
    Ok(())
}

pub(super) fn write_migrate_schema<I: Ord>(
    old_schema: &Schema<I>,
    new_schema: &Schema<I>,
    handler: &Option<Ident<I>>,
    new_path: &String,
) -> GenResult<String> {
    let old_schema_name = old_schema.version.replace(".", "_");
    let new_schema_name = new_schema.version.replace(".", "_");
    let new_schema_full_path = format!("{}::{}", new_path, new_schema_name);
    let migration_handler = handler.as_ref().map_or_else(
        || "DefaultMigrationHandler".to_string(),
        |handler| handler.to_string(),
    );

    let new_types: HashSet<_> = new_schema.iter().map(SchemaStm::get_type).collect();

    let mut s = String::new();
    writeln!(s, "impl<NK, EK> MigrateSchema<NK, EK, {new_schema_full_path}<NK, EK>> for {old_schema_name}<NK, EK>")?;
    writeln!(s, "where")?;
    writeln!(s, "    NK: Key,")?;
    writeln!(s, "    EK: Key")?;
    writeln!(s, "{{")?;
    writeln!(s, "    fn update_node(&self, _new_schema: &{new_schema_full_path}<NK, EK>, node: Self::N) -> SchemaResult<Option<<{new_schema_full_path}<NK, EK> as SchemaExt<NK, EK>>::N>, NK, EK, {new_schema_full_path}<NK, EK>> {{")?;
    writeln!(s, "        match node {{")?;
    for n in old_schema.nodes() {
        let node_type = &n.name;

        if new_types.contains(&n.name) {
            writeln!(s, "            Node::{node_type}(e) => Ok(Some({new_path}::Node::{node_type}(e.try_into().map_err(|e: UpgradeError| SchemaError::<NK, EK, {new_schema_full_path}<NK, EK>>::UpgradeError(e))?))),")?;
        } else {
            writeln!(s, "            Node::{node_type}(_) => Ok(None),")?;
        }
    }
    writeln!(s, "           #[allow(unreachable_patterns)]")?;
    writeln!(s, "           _ => Ok(None)")?;
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "")?;
    writeln!(s, "    fn update_edge(&self, _new_schema: &{new_schema_full_path}<NK, EK>, edge: Self::E) -> SchemaResult<Option<<{new_schema_full_path}<NK, EK> as SchemaExt<NK, EK>>::E>, NK, EK, {new_schema_full_path}<NK, EK>> {{")?;
    writeln!(s, "        match edge {{")?;
    for e in old_schema.edges() {
        let edge_type = &e.name;

        if new_types.contains(&e.name) {
            writeln!(s, "            Edge::{edge_type}(e) => Ok(Some({new_path}::Edge::{edge_type}(e.try_into().map_err(|e: UpgradeError| SchemaError::<NK, EK, {new_schema_full_path}<NK, EK>>::UpgradeError(e))?))),")?;
        } else {
            writeln!(s, "            Edge::{edge_type}(_) => Ok(None),")?;
        }
    }
    writeln!(s, "            #[allow(unreachable_patterns)]")?;
    writeln!(s, "            _ => Ok(None)")?;
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "")?;
    writeln!(s, "     fn update_edge_type(&self, _new_schema: &{new_schema_full_path}<NK, EK>, edge_type: <Self::E as Typed>::Type) -> Option<<<{new_schema_full_path}<NK, EK> as SchemaExt<NK, EK>>::E as Typed>::Type> {{")?;
    writeln!(s, "        match edge_type {{")?;
    for n in old_schema.edges() {
        let node_type = &n.name;

        if new_types.contains(&n.name) {
            writeln!(
                s,
                "            EdgeType::{node_type} => Some({new_path}::EdgeType::{node_type}),"
            )?;
        } else {
            writeln!(s, "            EdgeType::{node_type} => None,")?;
        }
    }
    writeln!(s, "            #[allow(unreachable_patterns)]")?;
    writeln!(s, "            _ => None")?;
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "")?;
    writeln!(s, "     fn update_node_type(&self, _new_schema: &{new_schema_full_path}<NK, EK>, node_type: <Self::N as Typed>::Type) -> Option<<<{new_schema_full_path}<NK, EK> as SchemaExt<NK, EK>>::N as Typed>::Type> {{")?;
    writeln!(s, "        match node_type {{")?;
    for n in old_schema.nodes() {
        let node_type = &n.name;

        if new_types.contains(&n.name) {
            writeln!(
                s,
                "            NodeType::{node_type} => Some({new_path}::NodeType::{node_type}),"
            )?;
        } else {
            writeln!(s, "            NodeType::{node_type} => None,")?;
        }
    }
    writeln!(s, "            #[allow(unreachable_patterns)]")?;
    writeln!(s, "            _ => None")?;
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;
    writeln!(s, "")?;
    writeln!(s, "impl<NK, EK> Migration<NK, EK, {new_schema_full_path}<NK, EK>> for {old_schema_name}<NK, EK>")?;
    writeln!(s, "where")?;
    writeln!(s, "    NK: Key,")?;
    writeln!(s, "    EK: Key")?;
    writeln!(s, "{{")?;
    writeln!(s, "    type Handler = {migration_handler};")?;
    writeln!(s, "}}")?;
    writeln!(s, "")?;
    writeln!(s, "impl<NK, EK> DirectMigration<NK, EK, {new_schema_full_path}<NK, EK>> for {old_schema_name}<NK, EK>")?;
    writeln!(s, "where")?;
    writeln!(s, "    NK: Key,")?;
    writeln!(s, "    EK: Key")?;
    writeln!(s, "{{")?;
    writeln!(s, "    fn migrate(g: TypedGraph<NK, EK, Self>) -> GenericTypedResult<TypedGraph<NK, EK, {new_schema_full_path}<NK, EK>>, NK, EK> {{")?;
    writeln!(s, "        <Self as Migration<NK, EK, _>>::migrate(g, &{migration_handler}, Default::default())")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;
    Ok(s)
}
