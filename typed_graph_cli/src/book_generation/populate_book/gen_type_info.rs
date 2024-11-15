
use build_script_lang::schema::{Schema, SchemaStm};
use build_script_shared::parsers::Types;
use build_script_shared::InputMarker;

use crate::{GenResult};

pub fn gen_type_def(ty: &Types<InputMarker<String>>, ref_name: &str, schema: &Schema<InputMarker<String>>) -> GenResult<String> {
    match ty {
        Types::String(_) => Ok("<a href=\"../../primitives.md#string\">String</a>".to_string()),
        Types::Bool(_) => Ok("<a href=\"../../primitives.md#bool\">bool</a>".to_string()),
        Types::F64(_) => Ok("<a href=\"../../primitives.md#f32-f64\">f64</a>".to_string()),
        Types::F32(_) => Ok("<a href=\"../../primitives.md#f32-f64\">f32</a>".to_string()),
        Types::Usize(_) => Ok("<a href=\"../../primitives.md#u8-u16-u32-u64-usize\">usize</a>".to_string()),
        Types::U64(_) => Ok("<a href=\"../../primitives.md#u8-u16-u32-u64-usize\">u64</a>".to_string()),
        Types::U32(_) => Ok("<a href=\"../../primitives.md#u8-u16-u32-u64-usize\">u32</a>".to_string()),
        Types::U16(_) => Ok("<a href=\"../../primitives.md#u8-u16-u32-u64-usize\">u16</a>".to_string()),
        Types::U8(_) => Ok("<a href=\"../../primitives.md#u8-u16-u32-u64-usize\">u8</a>".to_string()),
        Types::Isize(_) => Ok("<a href=\"../../primitives.md#i8-i16-i32-i64-isize\">isize</a>".to_string()),
        Types::I64(_) => Ok("<a href=\"../../primitives.md#i8-i16-i32-i64-isize\">i64</a>".to_string()),
        Types::I32(_) => Ok("<a href=\"../../primitives.md#i8-i16-i32-i64-isize\">i32</a>".to_string()),
        Types::I16(_) => Ok("<a href=\"../../primitives.md#i8-i16-i32-i64-isize\">i16</a>".to_string()),
        Types::I8(_) => Ok("<a href=\"../../primitives.md#i8-i16-i32-i64-isize\">i8</a>".to_string()),
        Types::Option { inner, .. } => Ok(format!("<a href=\"../../primitives.md#optiont\">Option</a><{}>", gen_type_def(inner, ref_name, schema)?)),
        Types::List { inner, .. } => Ok(format!("<a href=\"../../primitives.md#listt\">List</a><{}>", gen_type_def(inner, ref_name, schema)?)),
        Types::Set { inner, .. } => Ok(format!("<a href=\"../../primitives.md#sett\">List</a><{}>", gen_type_def(inner, ref_name, schema)?)),
        Types::Map { key, value, .. } => Ok(format!("<a href=\"../../primitives.md#mapk-v\">Map</a><{}, {}>", gen_type_def(key, ref_name, schema)?, gen_type_def(value, ref_name, schema)?)),
        Types::Reference {
            inner,
            generics,
            ..
        } => {
            let inner_name = inner.to_string();
            let ref_type = schema.get_type(None, &inner_name);
            let source_ref = match ref_type {
                Some(SchemaStm::Node(_)) => format!("<a href=\"../nodes/{inner_name}.md\">{inner_name}</a>"),
                Some(SchemaStm::Edge(_)) => format!("<a href=\"../edges/{inner_name}.md\">{inner_name}</a>"),
                Some(SchemaStm::Enum(_)) => format!("<a href=\"../types/{inner_name}.md\">{inner_name}</a>"),
                Some(SchemaStm::Import(_)) => format!("<a href=\"../imports.md#{}\">{inner_name}</a>", inner_name.to_lowercase()),
                Some(SchemaStm::Struct(_)) => format!("<a href=\"../structs/{inner_name}.md\">{inner_name}</a>"),
                None => format!("<a href=\"#{ref_name}\">{inner_name}</a>"),
            };

            if generics.is_empty() {
                Ok(format!("{source_ref}"))
            } else {
                Ok(format!("{source_ref}<{}>", generics.iter().map(|t| gen_type_def(t, ref_name, schema)).collect::<Result<Vec<_>, _>>()?.join(", ")))
            }
        }
    }
}