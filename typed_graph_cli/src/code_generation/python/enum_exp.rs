use build_script_lang::schema::{EnumExp, EnumVarient};
use std::fmt::Write;

use crate::{
    targets, CodeGenerator, GeneratedCode, ToDefaultPythonValue, ToPythonType, ToSnakeCase,
};

use super::write_comments;

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

        writeln!(s, "from __future__ import annotations")?;
        writeln!(s, "from typed_graph import NestedEnum")?;
        writeln!(
            s,
            "from typing import Optional, List, Set, Dict, TypeVar, Generic, ClassVar, Annotated, Literal, TYPE_CHECKING"
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
            writeln!(s, "class {enum_name}(NestedEnum):")?;
        } else {
            writeln!(
                s,
                "class {enum_name}(NestedEnum[{generic_refs}], Generic[{generic_refs}]):"
            )?;
        }

        write_comments(&mut s, &self.comments)?;

        if self.attributes.is_untagged() {
            writeln!(s, "    tagging: ClassVar[bool] = False")?;
        }

        if self.varients.is_empty() {
            write!(s, "    pass")?;
        }

        for varient in &self.varients {
            let name = varient.name();
            let comments = varient.comments();
            match varient {
                EnumVarient::Struct { fields, .. } => {
                    writeln!(s, "    {name} = {{")?;

                    if varient.attributes().is_untagged() {
                        writeln!(s, "         'tagging': Annotated[ClassVar[bool], False],")?;
                    }

                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        let field_type = field_value.field_type.to_python_type();

                        let mut field_attributes = Vec::new();

                        // Handle skipped
                        if field_value.attributes.is_skipped() {
                            field_attributes.push("exclude=True".to_string());

                            let default = field_value.field_type.to_default_python_value();
                            field_attributes.push(format!("default_factory=lambda:{default}"));
                        }

                        // Handle alias
                        let alias_attributes = field_value.attributes.get_alias();
                        if !alias_attributes.is_empty() {
                            let alias_literals = alias_attributes
                                .into_iter()
                                .map(|i| format!("'{i}'"))
                                .collect::<Vec<_>>()
                                .join(", ");
                            field_attributes.push(format!(
                                "validation_alias=AliasChoices('{field_name}', {alias_literals})"
                            ));
                        }

                        if !field_attributes.is_empty() {
                            writeln!(
                                s,
                                "         '{field_name}': Annotated[{field_type}, Field({})],",
                                field_attributes.join(", ")
                            )?;
                        } else {
                            writeln!(s, "         '{field_name}': {field_type},")?;
                        }
                    }
                    write!(s, "    }}")?;

                    let mut enum_attributes = Vec::new();

                    // Handle alias
                    let alias_attributes = varient.attributes().get_alias();
                    if !alias_attributes.is_empty() {
                        let alias_literals = alias_attributes
                            .into_iter()
                            .map(|i| format!("'{i}'"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        enum_attributes.push(format!(
                            "validation_alias=AliasChoices('{name}', '{alias_literals}')"
                        ));
                    }

                    if !enum_attributes.is_empty() {
                        write!(s, " = Field({})", enum_attributes.join(", "))?;
                    }
                }
                EnumVarient::Opaque { ty, attributes, .. } => {
                    let field_type = ty.to_python_type();

                    let mut field_attributes = Vec::new();

                    // Handle skipped
                    if attributes.is_skipped() {
                        field_attributes.push("exclude=True".to_string());

                        let default = ty.to_default_python_value();
                        field_attributes.push(format!("default_factory=lambda:{default}"));
                    }

                    // Handle alias
                    let alias_attributes = attributes.get_alias();
                    if !alias_attributes.is_empty() {
                        let alias_literals = alias_attributes
                            .into_iter()
                            .map(|i| format!("'{i}'"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        field_attributes.push(format!(
                            "validation_alias=AliasChoices('{name}', {alias_literals})"
                        ));
                    }

                    write!(s, "    {name} = {field_type}")?;
                }
                EnumVarient::Unit { .. } => {
                    write!(s, "    {name} = Literal")?;
                }
            }

            writeln!(s)?;

            if comments.has_doc() {
                writeln!(s, "    \"\"\"")?;
                for comment in comments.iter_doc() {
                    writeln!(s, "     {comment}")?;
                }
                writeln!(s, "    \"\"\"")?;
            }
        }

        let mut new_files = GeneratedCode::new();
        new_files.add_content(types_path, s);
        Ok(new_files)
    }
}
