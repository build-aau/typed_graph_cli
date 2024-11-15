use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

use crate::{
    targets, CodeGenerator, Direction, GenError, GenResult, GeneratedCode, Project, ToSnakeCase,
};

impl CodeGenerator<targets::Rust> for Project {
    fn get_filename(&self) -> String {
        "".to_string()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let output_folder = p.as_ref();
        if !output_folder.exists() || !output_folder.is_dir() {
            return Err(GenError::MissingFolder {
                folder: output_folder
                    .to_str()
                    .ok_or_else(|| GenError::MalformedPath)?
                    .to_string(),
            });
        }

        let mut new_files = GeneratedCode::new();

        for schema_id in self.iter_schema() {
            let schema = self.get_schema(schema_id)?;
            let added_files =
                CodeGenerator::<targets::Rust>::aggregate_content(schema, output_folder)?;
            new_files.append(added_files);
        }

        for changeset_id in self.iter_changesets() {
            let changeset = self.get_changeset(changeset_id)?;
            let new_schema = self.get_schema(&changeset.new_version)?;
            let old_schema = self.get_schema(&changeset.old_version)?;
            let added_files = CodeGenerator::<targets::Rust>::aggregate_content(
                &(changeset, old_schema, new_schema),
                output_folder,
            )?;
            new_files.append(added_files);
        }

        let added_files = write_transitive_migrations(self, output_folder)?;
        new_files.append(added_files);

        write_any_schema(&self, &mut new_files, output_folder)?;
        write_any_graph(&self, &mut new_files, output_folder)?;
        write_mod(self, &mut new_files, output_folder)?;

        Ok(new_files)
    }
}

fn write_mod(
    project: &Project,
    new_files: &mut GeneratedCode,
    project_folder: &Path,
) -> GenResult<()> {
    let project_mod_path = project_folder.join("mod.rs");

    let mut project_mod = String::new();

    writeln!(project_mod, "#[allow(unused)]")?;
    writeln!(project_mod, "mod imports;")?;
    writeln!(project_mod, "")?;

    for schema in project.iter_schema() {
        let schema = project.get_schema(schema)?;
        writeln!(
            project_mod,
            "pub mod {};",
            CodeGenerator::<targets::Rust>::get_filename(schema)
        )?;
        writeln!(project_mod, "#[allow(unused)]")?;
        writeln!(
            project_mod,
            "pub use {}::{};",
            CodeGenerator::<targets::Rust>::get_filename(schema),
            schema.version.to_string().replace(".", "_")
        )?;
    }

    writeln!(project_mod, "")?;
    writeln!(project_mod, "mod any_schema;")?;
    writeln!(project_mod, "pub use any_schema::*;")?;
    writeln!(project_mod, "mod any_graph;")?;
    writeln!(project_mod, "pub use any_graph::*;")?;

    let imports_path = project_folder.join("imports.rs");
    new_files.create_file(imports_path);

    new_files.add_content(project_mod_path, project_mod);

    Ok(())
}

fn write_any_schema(
    project: &Project,
    new_files: &mut GeneratedCode,
    project_folder: &Path,
) -> GenResult<()> {
    let mut s = String::new();

    writeln!(s, "use serde::{{Serialize, Deserialize}};")?;
    writeln!(s, "use super::*;")?;
    writeln!(s, "use typed_graph::*;")?;
    writeln!(s, "use std::str::FromStr;")?;
    writeln!(s, "")?;
    writeln!(s, "#[derive(Serialize, Deserialize, Clone, Debug)]")?;
    writeln!(s, "#[serde(untagged, bound = \"NK: Clone, EK: Clone\")]")?;    
    writeln!(s, "pub enum AnySchema<NK, EK> {{")?;
    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        writeln!(s, "    {schema_name}({schema_name}<NK, EK>),")?;
    }
    writeln!(s, "}}")?;
    writeln!(s, "")?;
    writeln!(
        s,
        "impl<NK, EK> FromStr for AnySchema<NK, EK> {{"
    )?;
    writeln!(
        s,
        "    type Err = GenericTypedError<String, String>;"
    )?;
    writeln!(s, "")?;
    writeln!(
        s,
        "    fn from_str(value: &str) -> Result<Self, Self::Err> {{"
    )?;
    let mut first = true;
    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        if first {
            first = false;
            write!(s, "        ")?;
        } else {
            write!(s, " else ")?;
        }
        writeln!(s, "if let Ok(v) = value.parse() {{")?;
        writeln!(s, "            Ok(AnySchema::{schema_name}(v))")?;
        write!(s, "        }}")?;
    }
    writeln!(s, " else {{")?;
    writeln!(
        s,
        "            Err(GenericTypedError::InvalidSchemaName(value.to_string(), \"AnySchema\".to_string()))"
    )?;
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;
    writeln!(s, "")?;
    writeln!(s, "impl<NK, EK> ToString for AnySchema<NK, EK> {{")?;
    writeln!(s, "    fn to_string(&self) -> String {{")?;
    writeln!(s, "        match self {{")?;
    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        writeln!(s, "            AnySchema::{schema_name}(s) => s.to_string(),")?;
    }
    writeln!(s, "        }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        writeln!(s, "")?;
        writeln!(s, "impl<NK, EK> From<{schema_name}<NK, EK>> for AnySchema<NK, EK> {{")?;
        writeln!(s, "    fn from(v: {schema_name}<NK, EK>) -> Self {{")?;
        writeln!(s, "        AnySchema::{schema_name}(v)")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;
    }
    
    let any_schema_path = project_folder.join("any_schema.rs");
    new_files.add_content(any_schema_path, s);

    Ok(())
}

fn write_any_graph(
    project: &Project,
    new_files: &mut GeneratedCode,
    project_folder: &Path,
) -> GenResult<()> {
    let mut s = String::new();

    writeln!(s, "use serde::de::{{DeserializeOwned, Error}};")?;
    writeln!(s, "use serde::{{Deserialize, Serialize}};")?;
    writeln!(s, "use serde_json::{{Map, Value}};")?;
    writeln!(s, "use super::*;")?;
    writeln!(s, "use std::convert::identity;")?;
    writeln!(s, "use typed_graph::any_graph::AnyWeight;")?;
    writeln!(s, "use typed_graph::{{DisAllowedEdge, DisAllowedNode, EdgeExt, Id, Key, NodeExt, SchemaExt, Typed, TypedGraph, TypedError, DirectMigration}};")?;
    writeln!(s, "")?;
    writeln!(s, "use crate::AnySchema;")?;
    writeln!(s, "")?;
    writeln!(s, "/// Generic container for any graph generated by typed_graph_cli  ")?;
    writeln!(s, "/// The container does not check for ")?;
    writeln!(s, "pub type AnyGraph<NK, EK> = TypedGraph<NK, EK, AnySchema<NK, EK>>;")?;
    writeln!(s, "")?;
    writeln!(s, "impl<NK: Key, EK: Key> SchemaExt<NK, EK> for AnySchema<NK, EK> {{")?;
    writeln!(s, "    type N = AnyWeight<NK>;")?;
    writeln!(s, "    type E = AnyWeight<EK>;")?;
    writeln!(s, "")?;
    writeln!(s, "    fn name(&self) -> String {{")?;
    writeln!(s, "        self.to_string()")?;
    writeln!(s, "    }}")?;
    writeln!(s, "")?;
    writeln!(s, "    fn allow_edge(")?;
    writeln!(s, "        &self,")?;
    writeln!(s, "        outgoing_edge_count: usize,")?;
    writeln!(s, "        incoming_edge_count: usize,")?;
    writeln!(s, "        edge_ty: <Self::E as typed_graph::Typed>::Type,")?;
    writeln!(s, "        source: <Self::N as typed_graph::Typed>::Type,")?;
    writeln!(s, "        target: <Self::N as typed_graph::Typed>::Type,")?;
    writeln!(s, "    ) -> Result<(), typed_graph::DisAllowedEdge> {{")?;
    writeln!(s, "        match self {{")?;
    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        writeln!(s, "            AnySchema::{schema_name}(schema) => {{")?;
        writeln!(s, "                schema.allow_edge(")?;
        writeln!(s, "                    outgoing_edge_count, ")?;
        writeln!(s, "                    incoming_edge_count, ")?;
        writeln!(s, "                    edge_ty.parse().map_err(|_| DisAllowedEdge::InvalidType)?, ")?;
        writeln!(s, "                    source.parse().map_err(|_| DisAllowedEdge::InvalidType)?, ")?;
        writeln!(s, "                    target.parse().map_err(|_| DisAllowedEdge::InvalidType)?")?;
        writeln!(s, "                )?")?;
        writeln!(s, "            }},")?;
    }
    writeln!(s, "        }}")?;
    writeln!(s, "        Ok(())")?;
    writeln!(s, "    }}")?;
    writeln!(s, "")?;
    writeln!(s, "    fn allow_node(&self, node_ty: <Self::N as typed_graph::Typed>::Type) -> Result<(), typed_graph::DisAllowedNode> {{")?;
    writeln!(s, "        match self {{")?;
    writeln!(s, "            AnySchema::V0_9_0(schema) => {{")?;
    writeln!(s, "                schema.allow_node(")?;
    writeln!(s, "                    node_ty.parse().map_err(|_| DisAllowedNode::InvalidType)?")?;
    writeln!(s, "                )?")?;
    writeln!(s, "            }},")?;
    writeln!(s, "            AnySchema::V1_0_0(schema) => {{")?;
    writeln!(s, "                schema.allow_node(")?;
    writeln!(s, "                    node_ty.parse().map_err(|_| DisAllowedNode::InvalidType)?")?;
    writeln!(s, "                )?")?;
    writeln!(s, "            }},")?;
    writeln!(s, "            AnySchema::V1_1_0(schema) => {{")?;
    writeln!(s, "                schema.allow_node(")?;
    writeln!(s, "                    node_ty.parse().map_err(|_| DisAllowedNode::InvalidType)?")?;
    writeln!(s, "                )?")?;
    writeln!(s, "            }}")?;
    writeln!(s, "        }}")?;
    writeln!(s, "        Ok(())")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    for schema_version in project.iter_schema() {
        let schema_name = schema_version.replace(".", "_");
        writeln!(s, "")?;
        writeln!(s, "impl<NK, EK> DirectMigration<NK, EK, {schema_name}<NK, EK>>  for AnySchema<NK, EK>")?;
        writeln!(s, "where ")?;
        writeln!(s, "    NK: Key + DeserializeOwned + Serialize, ")?;
        writeln!(s, "    EK: Key + DeserializeOwned + Serialize")?;
        writeln!(s, "{{")?;
        writeln!(s, "    fn migrate(")?;
        writeln!(s, "        g: typed_graph::TypedGraph<NK, EK, Self>,")?;
        writeln!(s, "    ) -> typed_graph::GenericTypedResult<typed_graph::TypedGraph<NK, EK, {schema_name}<NK, EK>>, NK, EK> {{")?;
        writeln!(s, "        let new_schema = {schema_name}::default();")?;
        writeln!(s, "        let new_schema_name = new_schema.to_string();")?;
        writeln!(s, "        let old_schema_name = g.get_schema().to_string();")?;
        writeln!(s, "        if new_schema_name != old_schema_name {{")?;
        writeln!(s, "            return Err(TypedError::InvalidSchemaName(new_schema_name, old_schema_name));")?;
        writeln!(s, "        }}")?;
        writeln!(s, "")?;
        writeln!(s, "        let new_g = g")?;
        writeln!(s, "            .update_schema(")?;
        writeln!(s, "                new_schema,")?;
        writeln!(s, "                |_current_schema, _new_schema, node| Ok(Some(serde_json::from_value(serde_json::to_value(&node)?)?)),")?;
        writeln!(s, "                |_current_schema, _new_schema, edge| Ok(Some(serde_json::from_value(serde_json::to_value(&edge)?)?)),")?;
        writeln!(s, "            )")?;
        writeln!(s, "            // filter_map returns an error for the new schema")?;
        writeln!(s, "            // So we have to convert it into an error for the joined schema")?;
        writeln!(s, "            .map_err(|e| e.map(identity, identity, |n| n.to_string(), |n| n.to_string()))?;")?;
        writeln!(s, "")?;
        writeln!(s, "        Ok(new_g)")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;
        writeln!(s, "")?;
        writeln!(s, "impl<NK, EK> DirectMigration<NK, EK, AnySchema<NK, EK>>  for {schema_name}<NK, EK>")?;
        writeln!(s, "where ")?;
        writeln!(s, "    NK: Key + DeserializeOwned + Serialize, ")?;
        writeln!(s, "    EK: Key + DeserializeOwned + Serialize")?;
        writeln!(s, "{{")?;
        writeln!(s, "    fn migrate(")?;
        writeln!(s, "        g: typed_graph::TypedGraph<NK, EK, Self>,")?;
        writeln!(s, "    ) -> typed_graph::GenericTypedResult<typed_graph::TypedGraph<NK, EK, AnySchema<NK, EK>>, NK, EK> {{")?;
        writeln!(s, "        let new_schema = g.get_schema().clone().into();")?;
        writeln!(s, "        let new_g = g")?;
        writeln!(s, "            .update_schema(")?;
        writeln!(s, "                new_schema,")?;
        writeln!(s, "                |_current_schema, _new_schema, node| Ok(Some(serde_json::to_value(&node)?.try_into()?)),")?;
        writeln!(s, "                |_current_schema, _new_schema, edge| Ok(Some(serde_json::to_value(&edge)?.try_into()?)),")?;
        writeln!(s, "            )")?;
        writeln!(s, "            // filter_map returns an error for the new schema")?;
        writeln!(s, "            // So we have to convert it into an error for the joined schema")?;
        writeln!(s, "            .map_err(|e| e.map(identity, identity, |n| n, |n| n))?;")?;
        writeln!(s, "        Ok(new_g)")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;
    }

    let any_graph_path = project_folder.join("any_graph.rs");
    new_files.add_content(any_graph_path, s);

    Ok(())
}

fn write_transitive_migrations(
    project: &Project,
    project_folder: &Path,
) -> GenResult<GeneratedCode> {
    let mut targets = HashSet::new();
    let mut sources = HashSet::new();
    let mut versions: HashMap<&String, HashSet<&String>> = HashMap::new();
    let version_iter = project.iter_version(Some(Direction::Backwards));

    for (source, target, _) in version_iter {
        targets.insert(target);
        sources.insert(source);
        versions.entry(source).or_default().insert(target);
    }

    let mut complete_paths = Vec::new();
    let mut paths: Vec<Vec<&String>> = sources.iter().map(|leaf| vec![*leaf]).collect();
    while let Some(mut path) = paths.pop() {
        let last = path.last().unwrap();
        if let Some(children) = versions.get(last) {
            for child in children {
                path.push(child);
                paths.push(path.clone());
                path.pop();
            }
        }
        if path.len() > 2 {
            complete_paths.push(path);
        }
    }

    let mut code = GeneratedCode::new();
    for mut path in complete_paths {
        let new_version = path.remove(0);
        path.reverse();
        let old_version = path.remove(0);

        let new_name = new_version.replace(".", "_");
        let new_mod = new_name.to_snake_case();
        let old_name = old_version.replace(".", "_");
        let old_mod = old_name.to_snake_case();
        let mut s = String::new();

        writeln!(s, "")?;
        writeln!(s, "impl<NK, EK> DirectMigration<NK, EK, super::super::{new_mod}::{new_name}<NK, EK>> for {old_name}<NK, EK>")?;
        writeln!(s, "where")?;
        writeln!(s, "    NK: Key,")?;
        writeln!(s, "    EK: Key,")?;
        writeln!(s, "{{")?;
        writeln!(s, "    fn migrate(")?;
        writeln!(s, "        g: TypedGraph<NK, EK, Self>")?;
        writeln!(s, "    ) -> GenericTypedResult<TypedGraph<NK, EK, super::super::{new_mod}::{new_name}<NK, EK>>, NK, EK> {{")?;

        write!(s, "        g")?;
        for version in path {
            let version_name = version.replace(".", "_");
            let version_mod = version_name.to_snake_case();
            write!(s, ".migrate_direct::<super::super::{version_mod}::{version_name}<_, _>>()?\n           ")?;
        }
        writeln!(s, ".migrate_direct()")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        let schema_path = project_folder.join(old_mod).join("schema.rs");
        code.add_content(schema_path, s);
    }

    Ok(code)
}
