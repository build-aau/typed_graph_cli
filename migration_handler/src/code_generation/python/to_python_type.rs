use build_script_shared::parsers::Types;

pub trait ToPythonType {
    fn to_python_type(&self) -> String;
}

impl ToPythonType for Types<String> {
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
            Types::Reference(ty) => ty.to_string(),
            Types::Option(ty, _) => format!("Optional[{}]", ty.to_python_type()),
            Types::List(ty, _) => format!("List[{}]", ty.to_python_type()),
            Types::Map(kty, vty, _) => format!("Dict[{}, {}]", kty.to_python_type(), vty.to_python_type()),
        }
    }
}
