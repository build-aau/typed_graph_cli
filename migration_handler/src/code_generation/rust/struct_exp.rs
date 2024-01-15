use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{StructExp, Visibility};
use std::fmt::Write;

use crate::{CodeGenerator, GeneratedCode, GenResult, ToSnakeCase, ToRustType, targets};

impl CodeGenerator<targets::Rust> for StructExp<String> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(&self, p: P) -> crate::GenResult<GeneratedCode> {
        let node_path = p.as_ref().join(format!("{}.rs", CodeGenerator::<targets::Rust>::get_filename(self)));
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
        writeln!(s, "pub struct {} {{", self.name)?;
        for (name, field_value) in &self.fields.fields {
            for comment in field_value.comments.iter_doc() {
                writeln!(s, "   /// {comment}")?;
            }
            let vis = match field_value.visibility {
                Visibility::Local => "pub(crate) ",
                Visibility::Private => "",
                Visibility::Public => "pub "
            };
            let field_type = field_value.ty.to_rust_type();
            writeln!(s, "   {vis}{name}: {field_type},")?;
        }
        writeln!(s, "}}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(node_path, s);
        Ok(new_files)
    }
}

pub(super) fn write_struct_from(
    n: &StructExp<String>, 
    changeset: &ChangeSet<String>, 
    parent_ty: &String
) -> GenResult<String> {
    let struct_type = &n.name;

    let mut s = String::new();
    writeln!(s, "impl From<{parent_ty}> for {struct_type} {{")?;
    writeln!(s, "    fn from(other: {parent_ty}) -> Self {{")?;
    writeln!(s, "       {struct_type} {{")?;
    for (name, _) in &n.fields.fields {
        let field_path = FieldPath::new_path(n.name.clone(), vec![name.clone()]);
        let changes = changeset.get_changes(field_path);

        let is_removed = changes.iter().any(|c| matches!(c, SingleChange::AddedField(_)));

        if is_removed {
            writeln!(s, "           {name}: Default::default()")?;
        } else {
            writeln!(s, "           {name}: other.{name}.into(),")?;
        }
    }
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    Ok(s)
}