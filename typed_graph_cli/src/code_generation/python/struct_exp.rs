use build_script_lang::schema::StructExp;
use std::fmt::Write;

use crate::{targets, CodeGenerator, GeneratedCode, ToPythonType, ToSnakeCase};

impl<I> CodeGenerator<targets::Python> for StructExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let node_path = p.as_ref().join(format!(
            "{}.py",
            CodeGenerator::<targets::Python>::get_filename(self)
        ));
        let struct_name = &self.name;
        let mut s = String::new();

        writeln!(s, "from typed_graph import RustModel")?;
        writeln!(s, "from typing import Optional, List, Dict, TypeVar, Generic")?;
        writeln!(s, "from ..imports import *")?;
        writeln!(s, "from ...imports import *")?;
        writeln!(s)?;

        for generic in &self.generics.generics {
            let letter = &generic.letter;
            writeln!(s, "{letter} = TypeVar(\"{letter}\")")?;
        }
        let generic_refs = self.generics.generics
            .iter()
            .map(|gen| format!("{}", gen.letter))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(s)?;

        if generic_refs.is_empty() {
            writeln!(s, "class {struct_name}(RustModel):")?;
        } else {
            writeln!(s, "class {struct_name}(RustModel, Generic[{generic_refs}]):")?;
        }
        if self.comments.has_doc() {
            writeln!(s, "     \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "     {comment}")?;
            }
            writeln!(s, "     \"\"\"")?;
        }
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            let field_type = field_value.field_type.to_python_type();
            writeln!(s, "     {field_name}: {field_type}")?;
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
