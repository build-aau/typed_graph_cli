use std::fmt::Write;

use build_script_lang::schema::Fields;

use crate::{GenResult, ToDefaultPythonValue, ToPythonType};

use super::write_comments;

pub fn write_fields<I>(s: &mut impl Write, fields: &Fields<I>, quote_fields: bool) -> GenResult<()> {
    for field_value in fields.iter() {
        let field_name = &field_value.name;
        let field_type = field_value.field_type.to_python_type_quoted(quote_fields);

        let mut field_attributes = Vec::new();

        // Handle untagged
        if field_value.attributes.is_skipped() {
            field_attributes.push("exclude=True".to_string());
        }

        if field_value.attributes.is_skipped() || field_value.attributes.is_default() {
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
                "validation_alias=AliasChoices('{field_name}', '{alias_literals}')"
            ));
        }

        if !field_attributes.is_empty() {
            writeln!(
                s,
                "    {field_name}: {field_type} = Field({})",
                field_attributes.join(", ")
            )?;
        } else {
            writeln!(s, "    {field_name}: {field_type}")?;
        }

        write_comments(s, &field_value.comments)?;
    }

    Ok(())
}
