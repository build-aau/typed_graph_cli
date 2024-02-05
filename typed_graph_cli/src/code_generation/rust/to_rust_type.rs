use build_script_shared::parsers::Types;

pub trait ToRustType {
    fn to_rust_type(&self) -> String;
}

impl<I> ToRustType for Types<I> {
    fn to_rust_type(&self) -> String {
        match self {
            Types::String(_) => self.to_string(),
            Types::Usize(_) => self.to_string(),
            Types::Bool(_) => self.to_string(),
            Types::F64(_) => self.to_string(),
            Types::F32(_) => self.to_string(),
            Types::U64(_) => self.to_string(),
            Types::U32(_) => self.to_string(),
            Types::U16(_) => self.to_string(),
            Types::U8(_) => self.to_string(),
            Types::Isize(_) => self.to_string(),
            Types::I64(_) => self.to_string(),
            Types::I32(_) => self.to_string(),
            Types::I16(_) => self.to_string(),
            Types::I8(_) => self.to_string(),
            Types::Reference{inner, generics, ..} => {
                let generics = generics
                    .iter()
                    .map(|g| g.to_rust_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let generic_declaration = if generics.is_empty() {
                    "".to_string()
                } else {
                    format!("<{generics}>")
                };

                format!("{inner}{generic_declaration}")
            },
            Types::Option{inner, ..} => format!("Option<{}>", inner.to_rust_type()),
            Types::List{inner, ..} => format!("Vec<{}>", inner.to_rust_type()),
            Types::Map{key, value,..} => {
                format!("HashMap<{}, {}>", key.to_rust_type(), value.to_rust_type())
            }
        }
    }
}
