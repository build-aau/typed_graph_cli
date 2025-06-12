use build_script_lang::schema::StructExp;
use std::fmt::Write;

use crate::{targets, CodeGenerator, GeneratedCode, ToSnakeCase};

use super::{write_comments, write_fields};

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

        writeln!(s, "from __future__ import annotations")?;
        writeln!(s, "from typed_graph import RustModel")?;
        writeln!(
            s,
            "from typing import Optional, List, Set, Dict, TypeVar, Generic, ClassVar, TYPE_CHECKING"
        )?;
        writeln!(s, "from pydantic import Field, AliasChoices")?;
        writeln!(s, "")?;
        writeln!(s, "if TYPE_CHECKING:")?;
        writeln!(s, "    from ..imports import *")?;
        writeln!(s, "    from ...imports import *")?;
        writeln!(s, "    from ..structs import *")?;
        writeln!(s, "    from ..types import *")?;
        writeln!(s)?;
        for generic in &self.generics.generics {
            let letter = &generic.letter;
            writeln!(s, "{letter} = TypeVar(\"{letter}\")")?;
        }
        let generic_refs = self
            .generics
            .generics
            .iter()
            .map(|gen| format!("{}", gen.letter))
            .collect::<Vec<_>>()
            .join(", ");


        if generic_refs.is_empty() {
            writeln!(s, "class {struct_name}(RustModel):")?;
        } else {
            writeln!(
                s,
                "class {struct_name}(RustModel, Generic[{generic_refs}]):"
            )?;
        }

        write_comments(&mut s, &self.comments)?;

        if self.attributes.is_untagged() {
            writeln!(s, "    tagging: ClassVar[bool] = False")?;
        }

        if self.fields.is_empty() {
            write!(s, "     pass")?;
        }

        write_fields(&mut s, &self.fields, true)?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}
