use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{EnumExp, EnumVarient};
use std::collections::HashSet;
use std::fmt::Write;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

use super::create_generics;

impl<I> CodeGenerator<targets::Rust> for EnumExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let enum_name = &self.name;

        let types_path = p.as_ref().join(format!(
            "{}.rs",
            CodeGenerator::<targets::Rust>::get_filename(self)
        ));
        let mut s = String::new();
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::*;")?;
        writeln!(s, "use serde::{{Serialize, Deserialize}};")?;
        #[cfg(feature = "diff")]
        writeln!(s, "use changesets::Changeset;")?;

        let attributes = vec![
            "Clone".to_string(),
            "Debug".to_string(),
            "Serialize".to_string(),
            "Deserialize".to_string(),
            #[cfg(feature = "diff")]
            "Changeset".to_string(),
        ];
        let attribute_s = attributes.join(", ");

        let generics = if !self.generics.generics.is_empty() {
            let mut generics = String::new();

            write!(generics, "<")?;
            let mut first = true;
            for generic in &self.generics.generics {
                if !first {
                    write!(generics, ", ")?;
                } else {
                    first = false;
                }
                write!(generics, "{}", generic.letter)?;
            } 
            write!(generics, ">")?;
            generics
        } else {
            "".to_string()
        };

        writeln!(s, "")?;
        for comment in self.comments.iter_doc() {
            writeln!(s, "   /// {comment}")?;
        }
        writeln!(s, "#[derive({attribute_s})]")?;
        writeln!(s, "pub enum {enum_name}{generics} {{", )?;
        for varient in &self.varients {
            match varient {
                EnumVarient::Struct { 
                    name, 
                    comments, 
                    fields, 
                    .. 
                } => {
                    for comment in comments.iter_doc() {
                        writeln!(s, "    /// {comment}")?;
                    }
                    writeln!(s, "    {name} {{")?;
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;

                        for comment in field_value.comments.iter_doc() {
                            writeln!(s, "        /// {comment}")?;
                        }

                        let field_type = field_value.field_type.to_rust_type();
                        writeln!(s, "        {field_name}: {field_type},")?;
                    }
                    writeln!(s, "    }},")?;
                }
                EnumVarient::Unit { 
                    name, 
                    comments, 
                    .. 
                } => {
                    for comment in comments.iter_doc() {
                        writeln!(s, "    /// {comment}")?;
                    }
                    writeln!(s, "    {name},")?;
                }
            }
        }
        writeln!(s, "}}")?;
        writeln!(s, "")?;
        writeln!(s, "impl{generics} std::fmt::Display for {enum_name}{generics} {{")?;
        writeln!(
            s,
            "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
        )?;
        writeln!(s, "        match self {{")?;
        for varient in &self.varients {
            let name = varient.name();

            match varient {
                EnumVarient::Struct { .. } => {
                    writeln!(
                        s,
                        "            {enum_name}::{name}{{ .. }} => write!(f, \"{name}\"),"
                    )?;
                },
                EnumVarient::Unit { .. } => {
                    writeln!(
                        s,
                        "            {enum_name}::{name} => write!(f, \"{name}\"),"
                    )?;
                }
            }
        }
        writeln!(s, "        }}")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(types_path, s);
        Ok(new_files)
    }
}

pub(super) fn write_type_from<I: Clone + PartialEq + Ord + Default>(
    t: &EnumExp<I>,
    changeset: &ChangeSet<I>,
    parent_ty: &String,
) -> GenResult<String> {
    let (end_bracket, new_type_generics, old_type_generics, impl_generics) = create_generics(&t.name, &t.generics, changeset)?;

    // writeln!(s, "impl{impl_generics} From<{parent_ty}{old_type_generics}> for {struct_type}{new_type_generics} {end_bracket}")?;

    let enum_name = &t.name;
    let mut s = String::new();
    writeln!(s, "impl{impl_generics} From<{parent_ty}{old_type_generics}> for Option<{enum_name}{new_type_generics}> {end_bracket}")?;
    writeln!(s, "    fn from(other: {parent_ty}{old_type_generics}) -> Self {{")?;
    writeln!(s, "        match other {{")?;

    for varient in &t.varients {
        let field_path = FieldPath::new_path(t.name.clone(), vec![varient.name().clone()]);
        let changes = changeset.get_changes(field_path);

        let is_new = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedVarient(_)));

        if !is_new {
            match varient {
                EnumVarient::Struct { name, fields, .. } => {

                    // Figure out which fields exists both in the new and old version of the varient
                    let persistent_fields: HashSet<_> = fields
                        .iter()
                        .filter(|field_value| {
                            let field_name = &field_value.name;
                            let field_path = FieldPath::new_path(enum_name.clone(), vec![name.clone(), (*field_name).clone()]);
                            let changes = changeset.get_changes(field_path);
                            let is_removed = changes
                                .iter()
                                .any(|c| matches!(c, SingleChange::AddedField(_) | SingleChange::RemovedField(_)));
                            !is_removed
                        })
                        .map(|field_value| &field_value.name)
                        .collect();

                    // We then build up the match statement with all of the patterns
                    writeln!(s, "               {parent_ty}::{name} {{")?;
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        if persistent_fields.contains(field_name) {
                            writeln!(s, "                   {field_name},")?;
                        }
                    }
                    writeln!(s, "                   ..")?;
                    writeln!(
                        s,
                        "               }} => Some({enum_name}::{name} {{"
                    )?;
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        if persistent_fields.contains(field_name) {
                            writeln!(s, "                   {field_name}: {field_name}.into(),")?;
                        } else {
                            writeln!(s, "                   {field_name}: Default::default(),")?;
                        }
                    }
                    writeln!(s, "               }}),")?;
                }
                EnumVarient::Unit { name, .. } => {
                    writeln!(
                        s,
                        "               {parent_ty}::{name} => Some({enum_name}::{name}),"
                    )?;
                }
            }
        }
    }

    writeln!(s, "               _ => None,")?;
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;
    Ok(s)
}
