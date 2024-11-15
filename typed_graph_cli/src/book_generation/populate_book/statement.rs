use std::collections::HashMap;

use build_script_lang::schema::{EdgeExp, EnumExp, EnumVarient, NodeExp, Schema, SchemaStm, StructExp};
use build_script_shared::InputMarker;
use serde::Serialize;
use tera::{Context, Tera};

use crate::book_generation::SchemaDocContext;
use crate::{GenError, GenResult};

use super::{gen_schema_example, gen_type_def, gen_varient_example};

/// Build context for any statement
pub fn populate_statement_context(section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>, tmpl: &Tera) -> GenResult<()> {
    for stm in schema.iter() {
        match stm {
            SchemaStm::Node(n) => {
                let content = populate_node(tmpl, section, n, schema)?;
                section.add_node_section(n.name.to_string(), content)?;
            },
            SchemaStm::Edge(n) => {
                let content = populate_edge(tmpl, section, n, schema)?;
                section.add_edge_section(n.name.to_string(), content)?;
            },
            SchemaStm::Struct(n) => {
                let content = populate_struct(tmpl, section, n, schema)?;
                section.add_struct_section(n.name.to_string(), content)?;
            },
            SchemaStm::Enum(n) => {
                let content = populate_types(tmpl, section, n, schema)?;
                section.add_type_section(n.name.to_string(), content)?;
            },
            SchemaStm::Import(n) => {
                // Imports are all stored in the imports section_header
                // section.add_imports_section(n.name.to_string(), content)?;
            }
        }
    }
    
    Ok(())
}

/// Common struct for representing data for one field
#[derive(Serialize)]
struct FieldData {
    name: String,
    ty: String,
    doc_comments: String,
    comments: String
}

#[derive(Serialize)]
struct NodeEndpointData {
    edge: String,
    source: String,
    target: String,
    bounds: String,
}

/// Build context for a node type
pub fn populate_node(tmpl: &Tera, section: &mut SchemaDocContext, expr: &NodeExp<InputMarker<String>>, schema: &Schema<InputMarker<String>>) -> GenResult<String> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);
    let node_name = expr.name.to_string();
    let ref_name = create_ref_name(&node_name);
    ctx.insert("node_name", &node_name);

    ctx.insert("doc_comments", &expr.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"));
    ctx.insert("comments", &expr.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"));
    
    let mut fields = Vec::new();
    for field in expr.fields.iter() {
        fields.push(FieldData {
            name: field.name.to_string(),
            ty: gen_type_def(&field.field_type, &ref_name, schema)?,
            doc_comments: field.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
            comments: field.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
        })
    }
    ctx.insert("fields", &fields);

    let outgoing_edges: Vec<_> = schema.iter().filter_map(|stm| if let SchemaStm::Edge(e) = stm {
        let endpoints: Vec<_> = e.endpoints.iter().filter(|((source, _), _)| source == &expr.name).map(|(st, data)| (st, data, &e.name)).collect();
        if !endpoints.is_empty() {
            Some(endpoints)
        } else {
            None
        }
    } else {
        None
    }).flatten().collect();

    
    let mut outgoing_endpoints = Vec::new();
    for ((source, target), data, e) in outgoing_edges.iter() {
        outgoing_endpoints.push(NodeEndpointData {
            edge: format!("[{e}](../edges/{e}.md)"),
            source: format!("[{source}](../nodes/{source}.md)"),
            target: format!("[{target}](../nodes/{target}.md)"),
            bounds: data.outgoing_quantity.to_string(),
        });
    }
    ctx.insert("outgoing_endpoints", &outgoing_endpoints);

    let incoming_edges: Vec<_> = schema.iter().filter_map(|stm| if let SchemaStm::Edge(e) = stm {
        let endpoints: Vec<_> = e.endpoints.iter().filter(|((_, target), _)| target == &expr.name).map(|(st, data)| (st, data, &e.name)).collect();
        if !endpoints.is_empty() {
            Some(endpoints)
        } else {
            None
        }
    } else {
        None
    }).flatten().collect();

    
    let mut incoming_endpoints = Vec::new();
    for ((source, target), data, e) in incoming_edges.iter() {
        incoming_endpoints.push(NodeEndpointData {
            edge: format!("[{e}](../edges/{e}.md)"),
            source: format!("[{source}](../nodes/{source}.md)"),
            target: format!("[{target}](../nodes/{target}.md)"),
            bounds: data.incoming_quantity.to_string(),
        });
    }
    ctx.insert("incoming_endpoints", &incoming_endpoints);

    let scope = Default::default();
    let stm = schema.get_type(None, &expr.name).ok_or_else(|| GenError::UnknownReference { name: expr.name.to_string() })?;
    ctx.insert("example", &gen_schema_example(stm, 0, schema, &scope)?);

    let content = tmpl.render("statements/node.md", &ctx)?;
    Ok(content)
}

#[derive(Serialize)]
struct EndpointData {
    source: String,
    target: String,
    outgoing_bounds: String,
    incoming_bounds: String
}

/// Build context for a edge type
pub fn populate_edge(tmpl: &Tera, section: &mut SchemaDocContext, expr: &EdgeExp<InputMarker<String>>, schema: &Schema<InputMarker<String>>) -> GenResult<String> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);
    let edge_name = expr.name.to_string();
    let ref_name = create_ref_name(&edge_name);
    ctx.insert("edge_name", &edge_name);

    ctx.insert("doc_comments", &expr.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"));
    ctx.insert("comments", &expr.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"));
    
    let mut fields = Vec::new();
    for field in expr.fields.iter() {
        fields.push(FieldData {
            name: field.name.to_string(),
            ty: gen_type_def(&field.field_type, &ref_name, schema)?,
            doc_comments: field.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
            comments: field.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
        })
    }
    ctx.insert("fields", &fields);

    let mut endpoints = Vec::new();
    for ((source, target), data) in expr.endpoints.iter() {
        endpoints.push(EndpointData {
            source: format!("[{source}](../nodes/{source}.md)"),
            target: format!("[{target}](../nodes/{target}.md)"),
            outgoing_bounds: data.outgoing_quantity.to_string(),
            incoming_bounds: data.incoming_quantity.to_string(),
        });
    }
    ctx.insert("endpoints", &endpoints);

    let scope = Default::default();
    let stm = schema.get_type(None, &expr.name).ok_or_else(|| GenError::UnknownReference { name: expr.name.to_string() })?;
    ctx.insert("example", &gen_schema_example(stm, 0, schema, &scope)?);

    let content = tmpl.render("statements/edge.md", &ctx)?;
    Ok(content)
}

#[derive(Serialize)]
enum VarientType {
    Unit,
    Opaque,
    Struct,
}

/// Union of all fields for all varients  
/// This reduces the amount of matching needed to be done in the template
#[derive(Serialize)]
struct EnumData {
    name: String,
    varient_type: VarientType,
    root_ty: Option<String>,
    root_fields: Option<Vec<FieldData>>,
    doc_comments: String,
    comments: String,
    example: String
}

/// Build context for a enum type
pub fn populate_types(tmpl: &Tera, section: &mut SchemaDocContext, expr: &EnumExp<InputMarker<String>>, schema: &Schema<InputMarker<String>>) -> GenResult<String> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);
    let generics = expr.generics.generics.iter().map(|g| g.letter.to_string()).collect::<Vec<_>>().join(", ");
    let enum_name = if generics.is_empty() {
        expr.name.to_string()
    } else {
        format!("{}\\<{}\\>", expr.name.to_string(), generics)
    };

    let ref_name = create_ref_name(&enum_name);
    ctx.insert("enum_name", &enum_name);

    ctx.insert("doc_comments", &expr.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"));
    ctx.insert("comments", &expr.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"));

    // Setup scope for generating examples
    let mut scope = HashMap::new();
    for generic in &expr.generics.generics {
        scope.insert(generic.letter.to_string(), None);
    }
    
    let mut varients = Vec::new();
    for varient in &expr.varients {
        match varient {
            EnumVarient::Unit { name, comments, .. } => {
                varients.push(EnumData {
                    name: name.to_string(),
                    varient_type: VarientType::Unit,
                    root_ty: None,
                    root_fields: None,
                    doc_comments: comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    comments: comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    example: gen_varient_example(varient, 0, schema, &scope)?
                });
            },
            EnumVarient::Opaque { name, comments, ty, .. } => {
                varients.push(EnumData {
                    name: name.to_string(),
                    varient_type: VarientType::Opaque,
                    root_ty: Some(gen_type_def(ty, &ref_name, schema)?),
                    root_fields: None,
                    doc_comments: comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    comments: comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    example: gen_varient_example(varient, 0, schema, &scope)?
                });
            },
            EnumVarient::Struct { name, comments, fields, .. } => {
                let mut root_fields = Vec::new();
                for field in fields.iter() {
                    root_fields.push(FieldData {
                        name: field.name.to_string(),
                        ty: gen_type_def(&field.field_type, &ref_name, schema)?,
                        doc_comments: field.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
                        comments: field.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    })
                }
                
                varients.push(EnumData {
                    name: name.to_string(),
                    varient_type: VarientType::Struct,
                    root_ty: None,
                    root_fields: Some(root_fields),
                    doc_comments: comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    comments: comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
                    example: gen_varient_example(varient, 0, schema, &scope)?
                })
            }
        }
    }
    ctx.insert("varients", &varients);

    let content = tmpl.render("statements/type.md", &ctx)?;
    Ok(content)
}

/// Build context for a struct type
pub fn populate_struct(tmpl: &Tera, section: &mut SchemaDocContext, expr: &StructExp<InputMarker<String>>, schema: &Schema<InputMarker<String>>) -> GenResult<String> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);

    let generics = expr.generics.generics.iter().map(|g| g.letter.to_string()).collect::<Vec<_>>().join(", ");
    let struct_name = if generics.is_empty() {
        expr.name.to_string()
    } else {
        format!("{}\\<{}\\>", expr.name.to_string(), generics)
    };
    let ref_name = create_ref_name(&struct_name);
    ctx.insert("struct_name", &struct_name);

    ctx.insert("doc_comments", &expr.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"));
    ctx.insert("comments", &expr.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"));

    let mut fields = Vec::new();
    for field in expr.fields.iter() {
        fields.push(FieldData {
            name: field.name.to_string(),
            ty: gen_type_def(&field.field_type, &ref_name, schema)?,
            doc_comments: field.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
            comments: field.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
        })
    }
    ctx.insert("fields", &fields);

    // Setup scope for generating examples
    let mut scope = HashMap::new();
    for generic in &expr.generics.generics {
        scope.insert(generic.letter.to_string(), None);
    }

    let stm = schema.get_type(None, &expr.name).ok_or_else(|| GenError::UnknownReference { name: expr.name.to_string() })?;
    ctx.insert("example", &gen_schema_example(stm, 0, schema, &scope)?);

    let content = tmpl.render("statements/struct.md", &ctx)?;
    Ok(content)
}

fn create_ref_name(name: &str) -> String {
    name.replace("<", "").replace(">", "").replace("\\", "").replace(",", "").replace(" ", "-").replace(" ", "").to_lowercase()
}