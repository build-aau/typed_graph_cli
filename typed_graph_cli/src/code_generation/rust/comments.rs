use std::fmt::Write;

use build_script_shared::parsers::Comments;

use crate::GenResult;

use super::FieldFormatter;

pub fn write_comments(s: &mut impl Write, comments: &Comments, fmt: FieldFormatter) -> GenResult<()> {
    let spaces = fmt.create_indents();
    for comment in comments.iter_doc() {
        writeln!(s, "{spaces}/// {comment}")?;
    }

    Ok(())
}