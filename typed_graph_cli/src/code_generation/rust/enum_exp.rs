use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_lang::schema::{EnumExp, EnumVarient};
use std::collections::HashSet;
use std::fmt::Write;

use crate::{targets, CodeGenerator, GenResult, GeneratedCode, ToRustType, ToSnakeCase};

use super::{create_generics, write_comments, write_fields, FieldFormatter};

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
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use serde::{{Serialize, Deserialize, self}};")?;
        #[cfg(feature = "diff")]
        writeln!(s, "use changesets::Changeset;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use indexmap::IndexMap;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(s, "use std::collections::HashSet;")?;
        writeln!(s, "#[allow(unused_imports)]")?;
        writeln!(
            s,
            "use typed_graph::{{GenericTypedError, GenericTypedResult}};"
        )?;

        let mut derive_traits = vec![
            "Clone".to_string(),
            "Debug".to_string(),
            #[cfg(feature = "diff")]
            "Changeset".to_string(),
        ];

        if self.is_only_units() {
            derive_traits.push("Copy".to_string());
            derive_traits.push("Serialize".to_string());
            derive_traits.push("Deserialize".to_string());
            derive_traits.push("PartialEq".to_string());
            derive_traits.push("Eq".to_string());
            derive_traits.push("PartialOrd".to_string());
            derive_traits.push("Ord".to_string());
            derive_traits.push("Hash".to_string());
        }

        let derive_funcs = self.attributes.get_functions("derive");
        for derived in derive_funcs {
            for value in &derived.values {
                derive_traits.push(value.to_string());
            }
        }
        let derive_traits_s = derive_traits.join(", ");

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
        write_comments(&mut s, &self.comments, Default::default())?;

        writeln!(s, "#[derive({derive_traits_s})]")?;
        if self.attributes.is_untagged() {
            writeln!(s, "#[serde(untagged)]")?;
        }
        writeln!(s, "pub enum {enum_name}{generics} {{",)?;
        for varient in &self.varients {
            if varient.attributes().has_derived_default() {
                writeln!(s, "    #[default]")?;
            }

            match varient {
                EnumVarient::Struct {
                    name,
                    comments,
                    fields,
                    attributes,
                    ..
                } => {
                    write_comments(
                        &mut s,
                        comments,
                        FieldFormatter {
                            indents: 1,
                            include_visibility: false,
                        },
                    )?;

                    if attributes.is_skipped() {
                        writeln!(s, "    #[serde(skip)]")?;
                    }

                    if attributes.is_untagged() {
                        writeln!(s, "    #[serde(untagged)]")?;
                    }

                    let alias_attributes = attributes.get_alias();
                    if !alias_attributes.is_empty() {
                        let alias_literals = alias_attributes
                            .into_iter()
                            .map(|i| format!("alias=\"{i}\""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(s, "    #[serde({alias_literals})]")?;
                    }

                    writeln!(s, "    {name} {{")?;
                    write_fields(
                        &mut s,
                        fields,
                        FieldFormatter {
                            indents: 2,
                            include_visibility: false,
                        },
                    )?;
                    writeln!(s, "    }},")?;
                }
                EnumVarient::Opaque {
                    name,
                    comments,
                    attributes,
                    ty,
                    ..
                } => {
                    let field_type = ty.to_rust_type();
                    write_comments(
                        &mut s,
                        comments,
                        FieldFormatter {
                            indents: 1,
                            include_visibility: false,
                        },
                    )?;

                    if attributes.is_skipped() {
                        writeln!(s, "    #[serde(skip)]")?;
                    }

                    if attributes.is_untagged() {
                        writeln!(s, "    #[serde(untagged)]")?;
                    }

                    let alias_attributes = attributes.get_alias();
                    if !alias_attributes.is_empty() {
                        let alias_literals = alias_attributes
                            .into_iter()
                            .map(|i| format!("alias=\"{i}\""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(s, "    #[serde({alias_literals})]")?;
                    }

                    writeln!(s, "    {name} ({field_type}),")?;
                }
                EnumVarient::Unit {
                    name,
                    comments,
                    attributes,
                    ..
                } => {
                    write_comments(
                        &mut s,
                        comments,
                        FieldFormatter {
                            indents: 1,
                            include_visibility: false,
                        },
                    )?;

                    if attributes.is_skipped() {
                        writeln!(s, "    #[serde(skip)]")?;
                    }

                    if attributes.is_untagged() {
                        writeln!(s, "    #[serde(untagged)]")?;
                    }

                    let alias_attributes = attributes.get_alias();
                    if !alias_attributes.is_empty() {
                        let alias_literals = alias_attributes
                            .into_iter()
                            .map(|i| format!("alias=\"{i}\""))
                            .collect::<Vec<_>>()
                            .join(", ");
                        writeln!(s, "    #[serde({alias_literals})]")?;
                    }

                    writeln!(s, "    {name},")?;
                }
            }
        }
        writeln!(s, "}}")?;
        writeln!(s, "")?;
        writeln!(
            s,
            "impl{generics} std::fmt::Display for {enum_name}{generics} {{"
        )?;
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
                }
                EnumVarient::Opaque { .. } => {
                    writeln!(
                        s,
                        "            {enum_name}::{name}(_) => write!(f, \"{name}\"),"
                    )?;
                }
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
    let mut omit_convertion = false;

    if !t.generics.generics.is_empty() {
        // https://github.com/rust-lang/rust/issues/50133
        // TryFrom does not allow us to do generic implementations
        // So instead of making our own trait we just force the user to implement all needed types
        omit_convertion = true;
    }

    let (new_type_generics, old_type_generics) =
        create_generics(&t.name, &t.generics, changeset)?;

    // writeln!(s, "impl{impl_generics} From<{parent_ty}{old_type_generics}> for {struct_type}{new_type_generics} {end_bracket}")?;

    let enum_name = &t.name;
    let mut s = String::new();
    writeln!(s, "impl TryFrom<{parent_ty}{old_type_generics}> for {enum_name}{new_type_generics} {{")?;
    writeln!(s, "    type Error = GenericTypedError<String, String>;")?;
    writeln!(s, "")?;
    writeln!(
        s,
        "    fn try_from(other: {parent_ty}{old_type_generics}) -> GenericTypedResult<Self, String, String> {{",
    )?;
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
                            let field_path = FieldPath::new_path(
                                enum_name.clone(),
                                vec![name.clone(), (*field_name).clone()],
                            );
                            let changes = changeset.get_changes(field_path);
                            let is_removed = changes.iter().any(|c| {
                                matches!(
                                    c,
                                    SingleChange::AddedField(_) | SingleChange::RemovedField(_)
                                )
                            });
                            !is_removed
                        })
                        .map(|field_value| &field_value.name)
                        .collect();

                    // We then build up pattern in the match statement statement with all of the patterns
                    writeln!(s, "           {parent_ty}::{name} {{")?;
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        if persistent_fields.contains(field_name) {
                            writeln!(s, "                {field_name},")?;
                        }
                    }
                    writeln!(s, "                ..")?;
                    writeln!(s, "            }} => Ok({enum_name}::{name} {{")?;
                    // Lastly we implement the conversion
                    for field_value in fields.iter() {
                        let field_name = &field_value.name;
                        if persistent_fields.contains(field_name) {
                            let field_path = FieldPath::new_path(
                                enum_name.clone(),
                                vec![name.clone(), (*field_name).clone()],
                            );

                            let changes = changeset.get_changes(field_path);

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
                            let mut need_manual_implementation = false;
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
                                    writeln!(
                                        s,
                                        "                {field_name}: /* Insert convertion */,"
                                    )?;
                                } else {
                                    writeln!(
                                        s,
                                        "                {field_name}: {},",
                                        type_change
                                            .old_type()
                                            .gen_convertion(format!("{field_name}"), true, type_change.new_type())
                                    )?;
                                }
                            } else {
                                writeln!(s, "           {field_name}: {},", field_value.field_type.gen_convertion(format!("{field_name}"), true, &field_value.field_type))?;
                            }
                        } else {
                            writeln!(s, "                {field_name}: Default::default(),")?;
                        }
                    }
                    writeln!(s, "           }}),")?;
                }
                EnumVarient::Opaque { name, ty, .. } => {
                    let field_path = FieldPath::new_path(
                        enum_name.clone(),
                        vec![name.clone()],
                    );

                    let changes = changeset.get_changes(field_path);

                    let type_change = changes
                        .iter()
                        .filter_map(|c| {
                            if let SingleChange::EditedOpaque(v) = c {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .next();
                    
                    if let Some(type_change) = type_change {
                        writeln!(
                            s,
                            "            {parent_ty}::{name}(ty) => Ok({enum_name}::{name}({})),",
                            type_change.old_type().gen_convertion("ty".to_string(), true, type_change.new_type())
                        )?;
                    } else {
                        writeln!(
                            s,
                            "            {parent_ty}::{name}(ty) => Ok({enum_name}::{name}({})),",
                            ty.gen_convertion("ty".to_string(), true, ty)
                        )?;
                    }
                }
                EnumVarient::Unit { name, .. } => {
                    writeln!(
                        s,
                        "            {parent_ty}::{name} => Ok({enum_name}::{name}),"
                    )?;
                }
            }
        }
    }

    writeln!(s, "       }}")?;
    writeln!(s, "    }}")?;
    writeln!(s, "}}")?;

    if omit_convertion {
        Ok(format!("/*Requires manual implementation\n{s}*/"))
    } else {
        Ok(s)
    }
}
