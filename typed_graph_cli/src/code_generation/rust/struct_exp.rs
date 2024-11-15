use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::StructExp;
use std::fmt::Write;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

use super::{create_generics, write_comments, write_fields, FieldFormatter};

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
        writeln!(s, "use indexmap::IndexMap;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use std::collections::HashSet;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use typed_graph::*;")?;
        writeln!(s, "use serde::{{Serialize, Deserialize}};")?;
        #[cfg(feature = "diff")]
        writeln!(s, "use changesets::Changeset;")?;

        let mut derive_traits = vec![
            "Clone".to_string(),
            "Debug".to_string(),
            #[cfg(feature = "diff")]
            "Changeset".to_string(),
        ];

        let derive_funcs = self.attributes.get_functions("derive");
        for derived in derive_funcs {
            for value in &derived.values {
                derive_traits.push(value.to_string());
            }
        }
        let derive_traits_s = derive_traits.join(", ");

        let mut generics = String::new();
        if !self.generics.generics.is_empty() {
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
        }

        writeln!(s, "")?;
        write_comments(
            &mut s,
            &self.comments,
            FieldFormatter {
                indents: 0,
                include_visibility: true,
            },
        )?;
        writeln!(s, "#[derive({derive_traits_s})]")?;
        writeln!(s, "pub struct {}{generics} {{", self.name)?;
        write_fields(
            &mut s,
            &self.fields,
            FieldFormatter {
                indents: 1,
                include_visibility: true,
            },
        )?;
        writeln!(s, "}}")?;

        writeln!(s, "")?;
        writeln!(s, "#[allow(unused)]")?;
        writeln!(s, "impl{generics} {}{generics} {{", self.name)?;
        writeln!(s, "    pub fn new(")?;
        for field_value in self.fields.iter() {
            let field_type = &field_value.field_type.to_rust_type();
            let field_name = &field_value.name;
            writeln!(s, "       {field_name}: {field_type},")?;
        }
        writeln!(s, "")?;
        writeln!(s, "   ) -> Self {{")?;
        writeln!(s, "        Self {{")?;
        for field_value in self.fields.iter() {
            let field_name = &field_value.name;
            writeln!(s, "           {field_name},")?;
        }
        writeln!(s, "")?;
        writeln!(s, "        }}")?;
        writeln!(s, "    }}")?;
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
    
    let mut omit_convertion = false;
    let struct_type = &n.name;

    if !n.generics.generics.is_empty() {
        // https://github.com/rust-lang/rust/issues/50133
        // TryFrom does not allow us to do generic implementations
        // So instead of making our own trait we just force the user to implement all needed types
        omit_convertion = true;
    }

    let (new_type_generics, old_type_generics) =
        create_generics(&n.name, &n.generics, changeset)?;

    let mut s = String::new();
    writeln!(s, "impl TryFrom<{parent_ty}{old_type_generics}> for {struct_type}{new_type_generics} {{")?;
    writeln!(s, "    type Error = GenericTypedError<String, String>;")?;
    writeln!(s, "")?;
    writeln!(
        s,
        "    fn try_from(other: {parent_ty}{old_type_generics}) -> GenericTypedResult<Self, String, String> {{"
    )?;
    writeln!(s, "       Ok({struct_type} {{")?;
    for field_value in n.fields.iter() {
        let field_name = &field_value.name;
        let field_path = FieldPath::new_path(n.name.clone(), vec![field_name.clone()]);
        let changes = changeset.get_changes(field_path.clone());

        let is_news = changes
            .iter()
            .any(|c| matches!(c, SingleChange::AddedField(_)));
        if is_news {
            writeln!(s, "           {field_name}: Default::default(),")?;
        } else {
            let changed_type = changes
                .iter()
                .find(|c| matches!(c, SingleChange::EditedFieldType(_)));
            if let Some(SingleChange::EditedFieldType(changed_type)) = changed_type {
                writeln!(
                    s,
                    "           {field_name}: {},",
                    changed_type.old_type()
                        .gen_convertion(format!("other.{field_name}"), true, changed_type.new_type())
                )?;
            } else {
                let mut need_manual_implementation = false;
                let type_change = changes
                    .iter()
                    .filter_map(|c| {
                        if let SingleChange::EditedFieldType(v) = c {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .next();
                if let Some(type_change) = type_change {
                    if !type_change
                        .old_type()
                        .is_gen_compatible(type_change.new_type())
                    {
                        // We cannot trust the auto generated conversion so a manual one should be made instead
                        omit_convertion = true;
                        need_manual_implementation = true;
                    }
                    if need_manual_implementation {
                        writeln!(s, "           {field_name}: /* Insert convertion */,")?;
                    } else {
                        writeln!(
                            s,
                            "           {field_name}: {},",
                            type_change
                                .old_type()
                                .gen_convertion(format!("other.{field_name}"), true, type_change.new_type())
                        )?;
                    } 
                } else {
                    writeln!(
                        s,
                        "           {field_name}: {},",
                        field_value.field_type
                            .gen_convertion(format!("other.{field_name}"), true, &field_value.field_type)
                    )?;
                }
            }
        }
    }
    writeln!(s, "       }})")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    if omit_convertion {
        Ok(format!("/*Requires manual implementation\n{s}*/"))
    } else {
        Ok(s)
    }
}