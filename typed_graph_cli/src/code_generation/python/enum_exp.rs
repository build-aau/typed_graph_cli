use build_script_lang::schema::{EnumExp, EnumVarient};
use std::fmt::Write;

use crate::{targets, CodeGenerator, GeneratedCode, ToPythonType, ToSnakeCase};

impl<I> CodeGenerator<targets::Python> for EnumExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let enum_name = &self.name;

        let types_path = p.as_ref().join(format!(
            "{}.py",
            CodeGenerator::<targets::Python>::get_filename(self)
        ));
        let mut s = String::new();

        writeln!(s, "from typed_graph import NestedEnum")?;
        writeln!(s, "from typing import Optional, List, Dict, TypeVar, Generic")?;
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
            writeln!(s, "class {enum_name}(NestedEnum):")?;
        } else {
            writeln!(s, "class {enum_name}(NestedEnum, Generic[{generic_refs}]):")?;
        }

        if self.comments.has_doc() {
            writeln!(s, "     \"\"\"")?;
            for comment in self.comments.iter_doc() {
                writeln!(s, "     {comment}")?;
            }
            writeln!(s, "     \"\"\"")?;
        }

        for varient in &self.varients {
            let name = varient.name();
            let comments = varient.comments();
            match varient {
                EnumVarient::Struct { fields, .. } => {
                    writeln!(s, "     {name}: {{")?;
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        let field_type = field_value.field_type.to_python_type();
                        writeln!(s, "          '{field_name}': {field_type},")?;
                    }
                    write!(s, "     }}")?;
                }
                EnumVarient::Unit { .. } => {
                    write!(s, "     {name}: str")?;
                }
            }
            
            writeln!(s)?;

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
