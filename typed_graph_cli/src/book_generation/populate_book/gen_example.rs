use build_script_lang::schema::{EnumVarient, Fields, Schema, SchemaStm};
use build_script_shared::parsers::Types;
use build_script_shared::InputMarker;
use std::collections::HashMap;
use std::fmt::Write;

use crate::{GenError, GenResult};

pub fn gen_keyed_field_example(key: String, value: String, indent: usize) -> GenResult<String> {
    let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();
    let lower_indent_s = (0..indent).map(|_| "     ").collect::<String>();

    let mut s = String::new();
    writeln!(s, "{{")?;
    writeln!(s, "{indent_s}\"{key}\": {value}")?;
    write!(s, "{lower_indent_s}}}")?;
    Ok(s)
}

/// Generate json example for fields  
/// id_type can be used to insert an id field asking for the given type name
pub fn gen_field_example(fields: &Fields<InputMarker<String>>, indent: usize, id_type: Option<String>, schema: &Schema<InputMarker<String>>, scope: &HashMap<String, Option<Types<InputMarker<String>>>>) -> GenResult<String> {
    if fields.is_empty() && id_type.is_none() {
        return Ok("{}".to_string());
    }
    let mut s = String::new();

    let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();
    let lower_indent_s = (0..indent).map(|_| "     ").collect::<String>();

    writeln!(s, "{{")?;
    let field_count = fields.len();

    // Insert id field
    if let Some(id) = &id_type {
        write!(s, "{indent_s}\"id\": /* {id} */")?;
        if field_count > 0 {
            writeln!(s, ",")?;
        } else {
            writeln!(s, "")?;
        }
    }

    let mut current_count = 0;
    // Insert other fields
    for field in fields.iter() {
        write!(s, "{indent_s}\"{}\": {}", field.name, gen_type_example(&field.field_type, false, indent + 1, schema, scope)?)?;
        
        current_count += 1;
        if current_count != field_count {
            writeln!(s, ",")?;
        } else {
            writeln!(s, "")?;
        }

    }
    write!(s, "{lower_indent_s}}}")?;

    Ok(s)
}

/// Generate json example for a specific type  
/// This will also resolve any reference to generics in the given scope  
/// If the generic is None then it does not have a specific value and will just get an empty implementation
pub fn gen_type_example(ty: &Types<InputMarker<String>>, escaped: bool, indent: usize, schema: &Schema<InputMarker<String>>, scope: &HashMap<String, Option<Types<InputMarker<String>>>>) -> GenResult<String> {
    let quotes = if escaped {
        "\""
    } else {
        ""
    };

    match ty {
        Types::String(_) => Ok("\"Lorem ipsum dolor sit amet\"".to_string()),
        Types::Usize(_) => Ok(format!("{quotes}1234{quotes}")),
        Types::Bool(_) => Ok(format!("{quotes}false{quotes}")),
        Types::F64(_) => Ok(format!("{quotes}1234.567{quotes}")),
        Types::F32(_) => Ok(format!("{quotes}1234.567{quotes}")),
        Types::U64(_) => Ok(format!("{quotes}1234{quotes}")),
        Types::U32(_) => Ok(format!("{quotes}1234{quotes}")),
        Types::U16(_) => Ok(format!("{quotes}1234{quotes}")),
        Types::U8(_) => Ok(format!("{quotes}123{quotes}")),
        Types::Isize(_) => Ok(format!("{quotes}-1234{quotes}")),
        Types::I64(_) => Ok(format!("{quotes}-1234{quotes}")),
        Types::I32(_) => Ok(format!("{quotes}-1234{quotes}")),
        Types::I16(_) => Ok(format!("{quotes}-1234{quotes}")),
        Types::I8(_) => Ok(format!("{quotes}123{quotes}")),
        Types::Option { inner, .. } => Ok(gen_type_example(inner, false, indent, schema, scope)?),
        Types::List { inner, .. }
        | Types::Set { inner, .. } => {
            let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();
            let lower_indent_s = (0..indent).map(|_| "     ").collect::<String>();
            let example = gen_type_example(&inner, false, indent + 1, schema, scope)?;
            Ok(format!("[\n{indent_s}{example},\n{indent_s}...\n{lower_indent_s}]"))
        },
        Types::Map { key, value, .. } => {
            let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();
            let lower_indent_s = (0..indent).map(|_| "     ").collect::<String>();
            let example_key = gen_type_example(&key, true, indent, schema, scope)?;
            let example_value = gen_type_example(&value, false, indent + 1, schema, scope)?;
            Ok(format!("{{\n{indent_s}{example_key}: {example_value},\n{indent_s}...\n{lower_indent_s}}}"))
        },
        Types::Reference { inner, generics, ..} => {
            if let Some(generic_replacement) = scope.get(&inner.to_string()) {
                // The type refences a genreic value
                // So we use the genric implementation
                if let Some(generic_replacement) = generic_replacement {
                    gen_type_example(generic_replacement, false, indent, schema, scope)
                } else {
                    // No implementation is provided
                    Ok(format!("/* {inner} */"))
                }
            } else {
                let schema_ty = schema.get_type(None, inner).ok_or_else(|| GenError::UnknownReference { name: inner.to_string() })?;

                let stm_generics = match schema_ty {
                    SchemaStm::Edge(_) => None,
                    SchemaStm::Node(_) => None,
                    SchemaStm::Struct(expr) => Some(&expr.generics),
                    SchemaStm::Enum(expr) => Some(&expr.generics),
                    SchemaStm::Import(_) => None
                };

                // Create a new scope using the generics provided to the refence
                let mut local_scope = HashMap::default();
                if let Some(stm_generics) = stm_generics {
                    let generic_convertion = stm_generics.generics.iter().zip(generics.iter());
                    for (source, target) in generic_convertion {
                        let updated = expand_type(&target, scope)?;

                        match &updated {
                            Types::Reference { inner, .. } => {
                                if &source.letter == inner {
                                    // If we let the references to the generics though we will get a stack overflow
                                    local_scope.insert(source.letter.to_string(), None);
                                } else {
                                    local_scope.insert(source.letter.to_string(), Some(updated));
                                }
                            },
                            _ => {
                                local_scope.insert(source.letter.to_string(), Some(updated));
                            }
                        }
                        
                    }
                }

                gen_schema_example(schema_ty, indent, schema, &local_scope)
            }
        }
    }
}

/// Expand all generic references to the provided values in the scope  
/// This ensures that the type does not store any old references to the scope
pub fn expand_type(ty: &Types<InputMarker<String>>, scope: &HashMap<String, Option<Types<InputMarker<String>>>>) -> GenResult<Types<InputMarker<String>>> {
    match ty {
        Types::String(v) => Ok(Types::String(v.clone())),
        Types::Usize(v) => Ok(Types::Usize(v.clone())),
        Types::Bool(v) => Ok(Types::Bool(v.clone())),
        Types::F64(v) => Ok(Types::F64(v.clone())),
        Types::F32(v) => Ok(Types::F32(v.clone())),
        Types::U64(v) => Ok(Types::U64(v.clone())),
        Types::U32(v) => Ok(Types::U32(v.clone())),
        Types::U16(v) => Ok(Types::U16(v.clone())),
        Types::U8(v) => Ok(Types::U8(v.clone())),
        Types::Isize(v) => Ok(Types::Isize(v.clone())),
        Types::I64(v) => Ok(Types::I64(v.clone())),
        Types::I32(v) => Ok(Types::I32(v.clone())),
        Types::I16(v) => Ok(Types::I16(v.clone())),
        Types::I8(v) => Ok(Types::I8(v.clone())),
        Types::Option { inner, marker } => Ok(Types::Option { inner: Box::new(expand_type(&inner, scope)?), marker: marker.clone() }),
        Types::List { inner, marker } => Ok(Types::List { inner: Box::new(expand_type(&inner, scope)?), marker: marker.clone() }),
        Types::Set { inner, marker } => Ok(Types::Set { inner: Box::new(expand_type(&inner, scope)?), marker: marker.clone() }),
        Types::Map { key, value, marker } => Ok(Types::Map { key: Box::new(expand_type(&key, scope)?), value: Box::new(expand_type(&value, scope)?), marker: marker.clone() }),
        r @ Types::Reference {
            inner,
            generics: generics_ref,
            marker
        } => {
            if let Some(g) = scope.get(&inner.to_string()) {
                Ok(g.clone().unwrap_or_else(|| r.clone()))
            } else {
                // Expand generics so they no longer has any references to the provided scope
                let mut updated_generics = Vec::new();
                for generic in generics_ref {
                    let updated = expand_type(&generic, scope)?;
                    updated_generics.push(Box::new(updated));
                }

                Ok(Types::Reference { inner: inner.clone(), generics: updated_generics, marker: marker.clone() })
            }
        }
    }
}

/// Generate a new json example for a schema statement
pub fn gen_schema_example(stm: &SchemaStm<InputMarker<String>>, indent: usize, schema: &Schema<InputMarker<String>>, scope: &HashMap<String, Option<Types<InputMarker<String>>>>) -> GenResult<String> {
    match stm {
        SchemaStm::Import(expr) => {
            Ok(format!("/* {} body */", expr.name))
        },
        SchemaStm::Edge(expr) => {
            let weight = gen_keyed_field_example(expr.name.to_string(), gen_field_example(&expr.fields, indent + 2, Some("EdgeId".to_string()), schema, scope)?, indent + 1)?;
            
            let mut s = String::new();
            let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();
            let lower_indent_s = (0..indent).map(|_| "     ").collect::<String>();
            
            writeln!(s, "{{")?;
            writeln!(s, "{indent_s}\"weight\": {},", weight)?;
            writeln!(s, "{indent_s}\"source\": /* NodeId */,")?;
            writeln!(s, "{indent_s}\"target\": /* NodeId */")?;
            writeln!(s, "{lower_indent_s}}}")?;

            Ok(s)
        },
        SchemaStm::Node(expr) => {
            gen_keyed_field_example(expr.name.to_string(), gen_field_example(&expr.fields, indent + 1, Some("NodeId".to_string()), schema, scope)?, indent)
        },
        SchemaStm::Struct(expr) => {
            gen_field_example(&expr.fields, indent, None, schema, scope)
        },
        SchemaStm::Enum(expr) => {
            let mut s = String::new();
            let varient_count = expr.varients.len();
            let mut current_count = 0;
            let indent_s = (0..indent + 1).map(|_| "     ").collect::<String>();

            // We give examples of how to use all the varients
            for varient in &expr.varients {
                s.push_str(&gen_varient_example(varient, indent + 1, schema, scope)?);
                current_count += 1;
                if current_count != varient_count{
                    write!(s, "\n{indent_s}| ")?;
                }
            }

            Ok(s)
        }
    }
}

/// Generate json examples of using an enum varient
pub fn gen_varient_example(varient: &EnumVarient<InputMarker<String>>, indent: usize, schema: &Schema<InputMarker<String>>, scope: &HashMap<String, Option<Types<InputMarker<String>>>>) -> GenResult<String> {
    match varient {
        EnumVarient::Unit { name, .. } => Ok(format!("\"{name}\"")),
        EnumVarient::Opaque { name, ty,.. } => gen_keyed_field_example(name.to_string(), gen_type_example(ty, false, indent + 1, schema, scope)?, indent),
        EnumVarient::Struct { name, fields, .. } => gen_keyed_field_example(name.to_string(), gen_field_example(fields, indent + 1, None, schema, scope)?, indent),
    }
}