use std::fmt::Write;

use build_script_lang::schema::{Fields, Visibility};

use crate::{GenResult, ToRustType};

use super::write_comments;

#[derive(Default, Clone)]
pub struct FieldFormatter {
    pub indents: usize,
    pub include_visibility: bool
}

impl FieldFormatter {
    pub fn create_indents(&self) -> String {
        (0..self.indents).map(|_| "    ").collect()
    }
}

pub fn write_fields<I>(
    s: &mut impl Write, 
    fields: &Fields<I>,
    fmt: FieldFormatter
) -> GenResult<()> {
    let space = fmt.create_indents(); 

    for field_value in fields.iter() {
        let field_name = &field_value.name;
        write_comments(s, &field_value.comments, fmt.clone())?;

        if field_value.attributes.is_skipped() {
            writeln!(s, "{space}#[serde(skip)]")?;
        }

        let alias_attributes = field_value.attributes.get_alias();
        if !alias_attributes.is_empty() {
            let alias_literals = alias_attributes
                .into_iter()
                .map(|i| format!("alias=\"{i}\""))
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(s, "{space}#[serde({alias_literals})]")?;
        }

        let vis = match (fmt.include_visibility, field_value.visibility) {
            (true, Visibility::Local) => "pub(crate) ",
            (true, Visibility::Public) => "pub ",
            (false, _) => ""
        };

        let field_type = field_value.field_type.to_rust_type();
        writeln!(s, "{space}{vis}{field_name}: {field_type},")?;
    }

    Ok(())
}