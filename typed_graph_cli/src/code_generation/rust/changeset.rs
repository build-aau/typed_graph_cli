use std::collections::HashSet;
use std::fmt::Debug;
use std::path::Path;

use super::*;
use build_changeset_lang::*;
use build_script_lang::schema::*;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToSnakeCase};

impl<I> CodeGenerator<targets::Rust> for (&ChangeSet<I>, &Schema<I>, &Schema<I>) 
where
    I: Ord + Debug + Clone + PartialEq + Default
{
    fn get_filename(&self) -> String {
        CodeGenerator::<targets::Rust>::get_filename(self.1)
    }

    fn aggregate_content<P: AsRef<Path>>(&self, p: P) -> GenResult<GeneratedCode> {
        let changeset = self.0;
        let old_schema = self.1;
        let new_schema = self.2;
        let new_schema_folder = p
            .as_ref()
            .join(CodeGenerator::<targets::Rust>::get_filename(self.2));
        let old_schema_folder = p
            .as_ref()
            .join(CodeGenerator::<targets::Rust>::get_filename(self.1));

        let old_mod = changeset
            .old_version
            .to_string()
            .replace(".", "_")
            .to_snake_case();
        let new_mod = changeset
            .new_version
            .to_string()
            .replace(".", "_")
            .to_snake_case();

        let nodes_folder = new_schema_folder.join("nodes");
        let edges_folder = new_schema_folder.join("edges");
        let types_folder = new_schema_folder.join("types");
        let structs_folder = new_schema_folder.join("structs");

        let mut new_files = GeneratedCode::new();

        let old_types: HashSet<_> = old_schema.content.iter().map(SchemaStm::get_type).collect();

        for stm in &new_schema.content {
            // Check if the type is new
            if !old_types.contains(stm.get_type()) {
                continue;
            }

            let (folder, filename) = match stm {
                SchemaStm::Node(n) => (
                    &nodes_folder,
                    CodeGenerator::<targets::Rust>::get_filename(n),
                ),
                SchemaStm::Struct(n) => (
                    &structs_folder,
                    CodeGenerator::<targets::Rust>::get_filename(n),
                ),
                SchemaStm::Edge(e) => (
                    &edges_folder,
                    CodeGenerator::<targets::Rust>::get_filename(e),
                ),
                SchemaStm::Enum(t) => (
                    &types_folder,
                    CodeGenerator::<targets::Rust>::get_filename(t),
                ),
                SchemaStm::Import(_) => continue,
            };

            let parent_ty = format!("super::super::super::{}::{}", old_mod, stm.get_type());

            let s = match stm {
                SchemaStm::Node(n) => write_node_from(n, changeset, &parent_ty),
                SchemaStm::Struct(n) => write_struct_from(n, changeset, &parent_ty),
                SchemaStm::Edge(e) => write_edge_from(e, changeset, &parent_ty),
                SchemaStm::Enum(t) => write_type_from(t, changeset, &parent_ty),
                SchemaStm::Import(_) => unimplemented!(),
            }?;

            let path = folder.join(format!("{}.rs", filename));
            new_files.add_content(path, s);
        }

        let new_path = format!("super::super::{}", new_mod);
        let s = write_migrate_schema(old_schema, new_schema, &changeset.handler, &new_path)?;
        let path = old_schema_folder.join("schema.rs");
        new_files.add_content(path, s);

        Ok(new_files)
    }
}
