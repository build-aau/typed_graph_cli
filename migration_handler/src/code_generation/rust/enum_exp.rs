use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::EnumExp;
use std::fmt::Write;

use crate::{CodeGenerator, GeneratedCode, GenResult, ToSnakeCase, targets};

impl CodeGenerator<targets::Rust> for EnumExp<String> {
    fn get_filename(&self) -> String {
        self.name.to_string().to_snake_case()
    }

    fn aggregate_content<P: AsRef<std::path::Path>>(&self, p: P) -> crate::GenResult<GeneratedCode> {
        let enum_name = &self.name;

        let types_path = p.as_ref().join(format!("{}.rs", CodeGenerator::<targets::Rust>::get_filename(self)));
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

        writeln!(s, "")?;
        for comment in self.comments.iter_doc() {
            writeln!(s, "   /// {comment}")?;
        }
        writeln!(s, "#[derive({attribute_s})]")?;
        writeln!(s, "pub enum {enum_name} {{")?;
        for (name, comments) in &self.varients {
            for comment in comments.iter_doc() {
                writeln!(s, "   /// {comment}")?;
            }
            writeln!(s, "   {},", name)?;
        }
        writeln!(s, "}}")?;
        writeln!(s, "")?;
        writeln!(s, "impl std::fmt::Display for {enum_name} {{")?;
        writeln!(s, "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{")?;
        writeln!(s, "       match self {{")?;
        for (name, _) in &self.varients {
            writeln!(s, "           {enum_name}::{name} => write!(f, \"{name}\"),")?;
        }
        writeln!(s, "       }}")?;
        writeln!(s, "    }}")?;
        writeln!(s, "}}")?;

        let mut new_files = GeneratedCode::new();
        new_files.add_content(types_path, s);
        Ok(new_files)
    }
}

pub(super) fn write_type_from(
    t: &EnumExp<String>, 
    changeset: &ChangeSet<String>, 
    parent_ty: &String
) -> GenResult<String> {
    let enum_name = &t.name;
    let mut s = String::new();
    writeln!(s, "impl From<{parent_ty}> for Option<{enum_name}> {{")?;
    writeln!(s, "    fn from(other: {parent_ty}) -> Self {{")?;
    writeln!(s, "match other {{")?;

    for (varient, _) in &t.varients {
        let field_path = FieldPath::new_path(t.name.clone(), vec![varient.clone()]);
        let changes = changeset.get_changes(field_path);

        let is_new = changes.iter().any(|c| matches!(c, SingleChange::AddedVarient(_)));

        if !is_new {
            writeln!(s, "               {parent_ty}::{varient} => {enum_name}::{varient},")?;
        }
    }

    writeln!(s, "               _ => None,")?;
    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;
    Ok(s)
}