use build_script_lang::schema::EnumExp;
use std::fmt::Write;

use crate::{CodeGenerator, GeneratedCode, ToSnakeCase, targets};

impl CodeGenerator<targets::Python> for EnumExp<String> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(&self, p: P) -> crate::GenResult<GeneratedCode> {
        let enum_name = &self.name;

        let types_path = p.as_ref().join(format!("{}.py", CodeGenerator::<targets::Python>::get_filename(self)));
        let mut s = String::new();

        writeln!(s, "from typed_graph import StrEnum")?;
        writeln!(s, "")?;
        writeln!(s, "class {enum_name}(StrEnum):")?;
        if self.comments.has_doc() {
            writeln!(s, "     \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "     {comment}")?;
            }
            writeln!(s, "     \"\"\"")?;
        }
        for (name, comments) in &self.varients {
            writeln!(s, "     {name} = '{name}'")?;
            if comments.has_doc() {
                writeln!(s, "     \"\"\"")?;
                for comment in comments.iter_doc() {
                    writeln!(s, "     {comment}")?;
                }
                writeln!(s, "     \"\"\"")?;
            }
        }

        let mut new_files = GeneratedCode::new();
        new_files.add_content(types_path, s);
        Ok(new_files)
    }
}