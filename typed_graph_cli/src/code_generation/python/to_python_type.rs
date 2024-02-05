use build_script_shared::parsers::Types;

pub trait ToPythonType {
    fn to_python_type(&self) -> String;
}

impl<I> ToPythonType for Types<I> {
    fn to_python_type(&self) -> String {
        match self {
            Types::String(_) => "str".to_string(),
            Types::Usize(_) => "int".to_string(),
            Types::Bool(_) => "bool".to_string(),
            Types::F64(_) => "float".to_string(),
            Types::F32(_) => "float".to_string(),
            Types::U64(_) => "int".to_string(),
            Types::U32(_) => "int".to_string(),
            Types::U16(_) => "int".to_string(),
            Types::U8(_) => "int".to_string(),
            Types::Isize(_) => "int".to_string(),
            Types::I64(_) => "int".to_string(),
            Types::I32(_) => "int".to_string(),
            Types::I16(_) => "int".to_string(),
            Types::I8(_) => "int".to_string(),
            Types::Reference{inner, generics, ..} => {
                let generics = generics
                    .iter()
                    .map(|g| g.to_python_type())
                    .collect::<Vec<_>>()
                    .join(", ");

                let generic_declaration = if !generics.is_empty() {
                    format!("[{generics}]")
                } else {
                    "".to_string()
                };

                format!("{inner}{generic_declaration}")
            },
            Types::Option{inner, ..} => format!("Optional[{}]", inner.to_python_type()),
            Types::List{inner, ..} => format!("List[{}]", inner.to_python_type()),
            Types::Map{key, value, ..} => {
                format!("Dict[{}, {}]", key.to_python_type(), value.to_python_type())
            }
        }
    }
}
