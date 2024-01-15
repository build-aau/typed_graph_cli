use build_script_shared::parsers::Types;

pub trait ToRustType {
    fn to_rust_type(&self) -> String;
}

impl ToRustType for Types<String> {
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
            Types::Reference(ty) => ty.to_string(),
            Types::Option(ty, _) => format!("Option<{}>", ty.to_rust_type()),
            Types::List(ty, _) => format!("Vec<{}>", ty.to_rust_type()),
            Types::Map(kty, vty, _) => format!("HashMap<{}, {}>", kty.to_rust_type(), vty.to_rust_type()),
        }
    }
}
