use build_script_lang::schema::StructExp;
use std::fmt::Write;

use crate::{CodeGenerator, GeneratedCode, ToSnakeCase, targets, ToPythonType};

impl CodeGenerator<targets::Python> for StructExp<String> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(&self, p: P) -> crate::GenResult<GeneratedCode> {
        let node_path = p.as_ref().join(format!("{}.py", CodeGenerator::<targets::Python>::get_filename(self)));
        let struct_name = &self.name;
        let mut s = String::new();

        writeln!(s, "from typed_graph import RustModel")?;
        writeln!(s, "from typing import Optional, List, Dict")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s, "")?;
        writeln!(s, "class {struct_name}(RustModel):")?;
        if self.comments.has_doc() {
            writeln!(s, "     \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "     {comment}")?;
            }
            writeln!(s, "     \"\"\"")?;
        }
        for (name, field_value) in &self.fields.fields {
            let field_type = field_value.ty.to_python_type();
            writeln!(s, "     {name}: {field_type}")?;
            if field_value.comments.has_doc() {
                writeln!(s, "     \"\"\"")?;
                for comment in field_value.comments.iter_doc() {
                    writeln!(s, "     {comment}")?;
                }
                writeln!(s, "     \"\"\"")?;
            }
        }

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}