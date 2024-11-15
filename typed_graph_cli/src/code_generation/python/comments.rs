use std::fmt::Write;

use build_script_shared::parsers::Comments;

use crate::GenResult;

pub fn write_comments(s: &mut impl Write, comments: &Comments) -> GenResult<()> {
    if comments.has_doc() {
        writeln!(s, "    \"\"\"")?;
        for comment in comments.iter_doc() {
            writeln!(s, "    {comment}")?;
        }
        writeln!(s, "    \"\"\"")?;
    }
    Ok(())
}
