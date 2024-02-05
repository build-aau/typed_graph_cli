use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{StructExp, Visibility};
use build_script_shared::parsers::{Generics, Ident, Mark, Types};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

use super::{create_generics, get_generic_changes, get_generic_field_type_changes};

impl<I> CodeGenerator<targets::Rust> for StructExp<I> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> crate::GenResult<GeneratedCode> {
        let node_path = p.as_ref().join(format!(
            "{}.rs",
            CodeGenerator::<targets::Rust>::get_filename(self)
        ));
        let mut s = String::new();

        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::super::imports::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::imports::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use super::super::*;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use std::collections::HashMap;")?;
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

        writeln!(s, "")?;
        for comment in self.comments.iter_doc() {
            writeln!(s, "/// {comment}")?;
        }
        writeln!(s, "#[derive({attribute_s})]")?;
        write!(s, "pub struct {}", self.name)?;
        if !self.generics.generics.is_empty() {
            write!(s, "<")?;
            let mut first = true;
            for generic in &self.generics.generics {
                if !first {
                    write!(s, ", ")?;
                } else {
                    first = false;
                }
                write!(s, "{}", generic.letter)?;
            } 
            write!(s, ">")?;
        }
        writeln!(s, " {{", )?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            for comment in field_value.comments.iter_doc() {
                writeln!(s, "   /// {comment}")?;
            }
            let vis = match field_value.visibility {
                Visibility::Local => "pub(crate) ",
                Visibility::Public => "pub ",
            };
            let field_type = field_value.field_type.to_rust_type();
            writeln!(s, "   {vis}{field_name}: {field_type},")?;
        }
        writeln!(s, "}}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}

pub(super) fn write_struct_from<I: Clone + PartialEq + Ord + Default>(
    n: &StructExp<I>,
    changeset: &ChangeSet<I>,
    parent_ty: &String,
) -> GenResult<String> {
    let struct_type = &n.name;

    let (end_bracket, new_type_generics, old_type_generics, impl_generics) = create_generics(&n.name, &n.generics, changeset)?;

    let mut s = String::new();
    writeln!(s, "impl{impl_generics} From<{parent_ty}{old_type_generics}> for {struct_type}{new_type_generics} {end_bracket}")?;
    writeln!(s, "    fn from(other: {parent_ty}{old_type_generics}) -> Self {{")?;
    writeln!(s, "       {struct_type} {{")?;
    for field_value in n.fields.iter() {
        let field_name = &field_value.name;
        let field_path = FieldPath::new_path(n.name.clone(), vec![field_name.clone()]);
        let changes = changeset.get_changes(field_path);

        let is_removed = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedField(_)));

        if is_removed {
            writeln!(s, "           {field_name}: Default::default(),")?;
        } else {
            writeln!(s, "           {field_name}: other.{field_name}.into(),")?;
        }
    }
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    Ok(s)
}