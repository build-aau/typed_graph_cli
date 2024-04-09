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

    let imports_path = project_folder.join("imports.rs");
    new_files.create_file(imports_path);

    new_files.add_content(project_mod_path, project_mod);

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
        writeln!(s, "    NK: Key + Default,")?;
        writeln!(s, "    EK: Key + Default,")?;
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
