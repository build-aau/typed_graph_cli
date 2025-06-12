use build_script_shared::parsers::Types;

pub trait ToRustType<I> {
    fn to_rust_type(&self) -> String;
    fn gen_convertion(&self, self_var: String, root: bool, new_type: &Types<I>) -> String;
    fn is_gen_compatible(&self, other: &Types<I>) -> bool;
}

impl<I> ToRustType<I> for Types<I> {
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
            Types::Reference {
                inner, generics, ..
            } => {
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
            }
            Types::Option { inner, .. } => format!("Option<{}>", inner.to_rust_type()),
            Types::List { inner, .. } => format!("Vec<{}>", inner.to_rust_type()),
            Types::Set { inner, .. } => format!("HashSet<{}>", inner.to_rust_type()),
            Types::Map { key, value, .. } => {
                format!("IndexMap<{}, {}>", key.to_rust_type(), value.to_rust_type())
            }
        }
    }

    fn gen_convertion(&self, self_var: String, root: bool, new_type: &Types<I>) -> String {
        match (self, new_type) {
            (Types::String(_), Types::String(_))
            | (Types::Usize(_), Types::Usize(_))
            | (Types::Bool(_), Types::Bool(_))
            | (Types::F64(_), Types::F64(_))
            | (Types::F32(_), Types::F32(_))
            | (Types::U64(_), Types::U64(_))
            | (Types::U32(_), Types::U32(_))
            | (Types::U16(_), Types::U16(_))
            | (Types::U8(_), Types::U8(_))
            | (Types::Isize(_), Types::Isize(_))
            | (Types::I64(_), Types::I64(_))
            | (Types::I32(_), Types::I32(_))
            | (Types::I16(_), Types::I16(_))
            | (Types::I8(_), Types::I8(_)) => {
                if root {
                    self_var
                } else {
                    format!("Ok({self_var})")
                }
            }
            (Types::Reference { .. }, Types::Reference { .. }) => {
                if root {
                    format!("{self_var}.try_into()?")
                } else {
                    format!("{self_var}.try_into()")
                }
            }
            (Types::Option { inner: linner, .. }, Types::Option { inner: rinner, .. }) => {
                if root {
                    format!("{self_var}.map(|v| {}).map_or(Ok(None), |v: Result<_, UpgradeError>| v.map(Some))?", linner.gen_convertion("v".to_string(), false, rinner))
                } else {
                    format!("{self_var}.map(|v| {}).map_or(Ok(None), |v: Result<_, UpgradeError>| v.map(Some))", linner.gen_convertion("v".to_string(), false, linner))
                }
            }
            (t, Types::Option { inner, .. }) => {
                if root {
                    format!("Some({})", t.gen_convertion(self_var, true, inner))
                } else {
                    format!("Ok(Some({}))", t.gen_convertion(self_var, true, inner))
                }
            }
            (Types::List { inner: linner, .. }, Types::List { inner: rinner, .. })
            | (Types::Set { inner: linner, .. }, Types::Set { inner: rinner, .. })
            | (Types::List { inner: linner, .. }, Types::Set { inner: rinner, .. })
            | (Types::Set { inner: linner, .. }, Types::List { inner: rinner, .. }) => {
                if root {
                    format!("{self_var}.into_iter().map(|v| Ok({})).collect::<Result<_, UpgradeError>>()?", linner.gen_convertion("v".to_string(), true, rinner))
                } else {
                    format!("{self_var}.into_iter().map(|v| Ok({})).collect::<Result<_, UpgradeError>>()", linner.gen_convertion("v".to_string(), true, rinner))
                }
            }
            (Types::Map { key: lkey, value: lvalue, .. }, Types::Map { key: rkey, value: rvalue, .. }) => {
                if root {
                    format!("{self_var}.into_iter().map(|(k, v)| Ok(({}, {}))).collect::<Result<Vec<(_, _)>, UpgradeError>>()?.into_iter().collect()", lkey.gen_convertion("k".to_string(), true, rkey), lvalue.gen_convertion("v".to_string(), true, rvalue))
                } else {
                    format!("Ok({self_var}.into_iter().map(|(k, v)| Ok(({}, {}))).collect::<Result<Vec<(_, _)>, UpgradeError>>()?.into_iter().collect())", lkey.gen_convertion("k".to_string(), true, rkey), lvalue.gen_convertion("v".to_string(), true, rvalue))
                }
            }
            _ => format!("/* Requires manual implementation from {} to {} */", self, new_type)
        }
    }

    /// Check if the automatic convertion is possible between two types
    ///
    /// All number convertions are based on https://doc.rust-lang.org/src/core/convert/num.rs.html#295
    fn is_gen_compatible(&self, new_type: &Types<I>) -> bool {
        match (self, new_type) {
            (Types::String(_), Types::String(_))
            | (Types::Bool(_), Types::Bool(_))
            | (Types::F64(_), Types::F64(_))
            | (Types::F32(_), Types::F64(_))
            | (Types::F32(_), Types::F32(_))
            | (Types::F64(_), Types::F32(_))
            | (Types::U64(_), Types::U64(_))
            | (Types::U32(_), Types::U64(_))
            | (Types::U16(_), Types::U64(_))
            | (Types::U8(_), Types::U64(_))
            | (Types::I64(_), Types::U64(_))
            | (Types::I32(_), Types::U64(_))
            | (Types::I16(_), Types::U64(_))
            | (Types::I8(_), Types::U64(_))
            | (Types::U32(_), Types::U32(_))
            | (Types::U16(_), Types::U32(_))
            | (Types::U8(_), Types::U32(_))
            | (Types::I32(_), Types::U32(_))
            | (Types::I16(_), Types::U32(_))
            | (Types::I8(_), Types::U32(_))
            | (Types::U16(_), Types::U16(_))
            | (Types::U8(_), Types::U16(_))
            | (Types::I16(_), Types::U16(_))
            | (Types::I8(_), Types::U16(_))
            | (Types::U8(_), Types::U8(_))
            | (Types::I8(_), Types::U8(_))
            | (Types::Usize(_), Types::Usize(_))
            | (Types::U64(_), Types::Usize(_))
            | (Types::U32(_), Types::Usize(_))
            | (Types::U16(_), Types::Usize(_))
            | (Types::U8(_), Types::Usize(_))
            | (Types::I64(_), Types::Usize(_))
            | (Types::I32(_), Types::Usize(_))
            | (Types::I16(_), Types::Usize(_))
            | (Types::I8(_), Types::Usize(_))
            | (Types::Isize(_), Types::Isize(_))
            | (Types::U64(_), Types::Isize(_))
            | (Types::U32(_), Types::Isize(_))
            | (Types::U16(_), Types::Isize(_))
            | (Types::U8(_), Types::Isize(_))
            | (Types::I64(_), Types::Isize(_))
            | (Types::I32(_), Types::Isize(_))
            | (Types::I16(_), Types::Isize(_))
            | (Types::I8(_), Types::Isize(_))
            | (Types::I64(_), Types::I64(_))
            | (Types::I32(_), Types::I64(_))
            | (Types::I16(_), Types::I64(_))
            | (Types::I8(_), Types::I64(_))
            | (Types::U64(_), Types::I64(_))
            | (Types::U32(_), Types::I64(_))
            | (Types::U16(_), Types::I64(_))
            | (Types::U8(_), Types::I64(_))
            | (Types::I32(_), Types::I32(_))
            | (Types::I16(_), Types::I32(_))
            | (Types::I8(_), Types::I32(_))
            | (Types::U32(_), Types::I32(_))
            | (Types::U16(_), Types::I32(_))
            | (Types::U8(_), Types::I32(_))
            | (Types::I16(_), Types::I16(_))
            | (Types::I8(_), Types::I16(_))
            | (Types::U16(_), Types::I16(_))
            | (Types::U8(_), Types::I16(_))
            | (Types::I8(_), Types::I8(_))
            | (Types::U8(_), Types::I8(_)) => true,
            (Types::Reference { inner: inner1, .. }, Types::Reference { inner: inner2, .. }) if inner1 == inner2 => true,
            (Types::Option { inner: inner1, .. }, Types::Option { inner: inner2, .. }) if inner1.is_gen_compatible(&inner2) => true,
            (t, Types::Option { inner, .. }) if t.is_gen_compatible(inner) => true,
            (Types::List { inner: inner1, .. }, Types::List { inner: inner2, .. }) if inner1.is_gen_compatible(&inner2) => true,
            (Types::Set { inner: inner1, .. }, Types::Set { inner: inner2, .. }) if inner1.is_gen_compatible(&inner2) => true,
            (Types::Set { inner: inner1, .. }, Types::List { inner: inner2, .. }) if inner1.is_gen_compatible(&inner2) => true,
            (Types::List { inner: inner1, .. }, Types::Set { inner: inner2, .. }) if inner1.is_gen_compatible(&inner2) => true,
            (
                Types::Map {
                    key: key1,
                    value: value1,
                    ..
                },
                Types::Map {
                    key: key2,
                    value: value2,
                    ..
                },
            ) if key1.is_gen_compatible(&key2) && value1.is_gen_compatible(&value2) => true,

            // Fail for all other convertions
            (Types::String(_), _)
            | (Types::Usize(_), _)
            | (Types::Bool(_), _)
            | (Types::F64(_), _)
            | (Types::F32(_), _)
            | (Types::U64(_), _)
            | (Types::U32(_), _)
            | (Types::U16(_), _)
            | (Types::U8(_), _)
            | (Types::Isize(_), _)
            | (Types::I64(_), _)
            | (Types::I32(_), _)
            | (Types::I16(_), _)
            | (Types::I8(_), _)
            | (Types::Reference { .. }, _)
            | (Types::Option { .. }, _)
            | (Types::List { .. }, _)
            | (Types::Set { .. }, _)
            | (Types::Map { .. }, _) => false,
        }
    }
}
