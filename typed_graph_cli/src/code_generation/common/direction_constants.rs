use crate::Direction;

pub const fn search_dir(dir: Direction) -> &'static str {
    match dir {
        Direction::Forward => "outgoing",
        Direction::Backwards => "incoming",
    }
}

pub const fn function_suffix(dir: Direction) -> &'static str {
    match dir {
        Direction::Forward => "out",
        Direction::Backwards => "inc",
    }
}

pub const fn rename_attribute_name(dir: Direction) -> &'static str {
    match dir {
        Direction::Forward => "rename_out",
        Direction::Backwards => "rename_inc",
    }
}