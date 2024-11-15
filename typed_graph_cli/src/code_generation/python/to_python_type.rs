use build_script_shared::parsers::Types;

pub trait ToPythonType {
    fn to_python_type(&self) -> String {
        self.to_python_type_quoted(true)
    }
    fn to_python_type_quoted(&self, requires_quotes: bool) -> String;
}

impl<I> ToPythonType for Types<I> {
    fn to_python_type_quoted(&self, requires_quotes: bool) -> String {
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
            Types::Option { inner, .. } => {
                format!("Optional[{}]", inner.to_python_type_quoted(requires_quotes))
            }
            Types::List { inner, .. } => {
                format!("List[{}]", inner.to_python_type_quoted(requires_quotes))
            }
            Types::Set { inner, .. } => {
                format!("Set[{}]", inner.to_python_type_quoted(requires_quotes))
            }
            Types::Map { key, value, .. } => {
                format!(
                    "Dict[{}, {}]",
                    key.to_python_type_quoted(requires_quotes),
                    value.to_python_type_quoted(requires_quotes)
                )
            }
            Types::Reference {
                inner, generics, ..
            } => {
                let generics = generics
                    .iter()
                    .map(|g| g.to_python_type_quoted(false))
                    .collect::<Vec<_>>()
                    .join(", ");

                let generic_declaration = if !generics.is_empty() {
                    format!("[{generics}]")
                } else {
                    "".to_string()
                };

                if requires_quotes {
                    format!("'{inner}{generic_declaration}'")
                } else {
                    format!("{inner}{generic_declaration}")
                }
            }
        }
    }
}

pub trait ToDefaultPythonValue {
    fn to_default_python_value(&self) -> String;
}

impl<I> ToDefaultPythonValue for Types<I> {
    fn to_default_python_value(&self) -> String {
        match self {
            Types::String(_) => "''".to_string(),
            Types::Usize(_) => "0".to_string(),
            Types::Bool(_) => "False".to_string(),
            Types::F64(_) => "0.0".to_string(),
            Types::F32(_) => "0.0".to_string(),
            Types::U64(_) => "0".to_string(),
            Types::U32(_) => "0".to_string(),
            Types::U16(_) => "0".to_string(),
            Types::U8(_) => "0".to_string(),
            Types::Isize(_) => "0".to_string(),
            Types::I64(_) => "0".to_string(),
            Types::I32(_) => "0".to_string(),
            Types::I16(_) => "0".to_string(),
            Types::I8(_) => "0".to_string(),
            Types::Option { .. } => "None".to_string(),
            Types::List { .. }
            | Types::Set { .. } => "[]".to_string(),
            Types::Map { .. } => "{{}}".to_string(),
            Types::Reference {
                inner, generics, ..
            } => {
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

                format!("guess_default({inner}{generic_declaration})")
            }
        }
    }
}
