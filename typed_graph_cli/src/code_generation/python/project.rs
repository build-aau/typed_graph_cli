use std::path::Path;

use crate::{targets, CodeGenerator, GenError, GenResult, GeneratedCode, Project};

impl CodeGenerator<targets::Python> for Project {
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
                CodeGenerator::<targets::Python>::aggregate_content(schema, output_folder)?;
            new_files.append(added_files);
        }

        write_init(&mut new_files, output_folder)?;

        Ok(new_files)
    }
}

fn write_init(new_files: &mut GeneratedCode, project_folder: &Path) -> GenResult<()> {
    let project_init_path = project_folder.join("__init__.py");
    new_files.create_file(project_init_path);

    let imports_path = project_folder.join("imports.py");
    new_files.create_file(imports_path);

    Ok(())
}
