use std::fs::File;
use std::io::Write;
use build_script_lang::schema::Schema;
use build_script_shared::InputMarker;
use serde::Serialize;
use tera::{Context, Tera};

use crate::book_generation::SchemaDocContext;
use crate::GenResult;

pub fn populate_section_header(section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>, tmpl: &Tera) -> GenResult<()> {
    populate_nodes_section_header(tmpl, section, schema)?;
    populate_edges_section_header(tmpl, section, schema)?;
    populate_structs_section_header(tmpl, section, schema)?;
    populate_types_section_header(tmpl, section, schema)?;
    populate_imports_section_header(tmpl, section, schema)?;

    Ok(())
}

pub fn populate_nodes_section_header(tmpl: &Tera, section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>) -> GenResult<()> {
    let mut ctx = Context::new();

    ctx.insert("section", &section);

    let mut node_types = Vec::new();
    for node in schema.nodes() {
        node_types.push(node.name.to_string());
    }
    ctx.insert("nodes", &node_types);

    let content = tmpl.render("section_headers/nodes.md", &ctx)?;
    let mut f = File::create(section.schema_path.join("nodes.md"))?;
    write!(f, "{}", content)?;
    Ok(())
}

pub fn populate_edges_section_header(tmpl: &Tera, section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>) -> GenResult<()> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);

    let mut edge_types = Vec::new();
    for edge in schema.edges() {
        edge_types.push(edge.name.to_string());
    }
    ctx.insert("edges", &edge_types);
    
    let content = tmpl.render("section_headers/edges.md", &ctx)?;
    let mut f = File::create(section.schema_path.join("edges.md"))?;
    write!(f, "{}", content)?;
    Ok(())
}

pub fn populate_structs_section_header(tmpl: &Tera, section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>) -> GenResult<()> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);

    let mut struct_types = Vec::new();
    for struct_ty in schema.structs() {
        struct_types.push(struct_ty.name.to_string());
    }
    ctx.insert("structs", &struct_types);
    
    let content = tmpl.render("section_headers/structs.md", &ctx)?;
    let mut f = File::create(section.schema_path.join("structs.md"))?;
    write!(f, "{}", content)?;
    Ok(())
}

pub fn populate_types_section_header(tmpl: &Tera, section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>) -> GenResult<()> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);

    let mut types = Vec::new();
    for e in schema.enums() {
        types.push(e.name.to_string());
    }
    ctx.insert("types", &types);
    
    let content = tmpl.render("section_headers/types.md", &ctx)?;
    let mut f = File::create(section.schema_path.join("types.md"))?;
    write!(f, "{}", content)?;
    Ok(())
}

#[derive(Serialize)]
struct ImportData {
    name: String,
    doc_comments: String,
    comments: String,
}

pub fn populate_imports_section_header(tmpl: &Tera, section: &mut SchemaDocContext, schema: &Schema<InputMarker<String>>) -> GenResult<()> {
    let mut ctx = Context::new();
    
    ctx.insert("section", &section);

    let mut imports = Vec::new();
    for import in schema.imports() {
        imports.push(ImportData {
            name: import.name.to_string(),
            doc_comments: import.comments.iter_doc().cloned().collect::<Vec<_>>().join("  \n"),
            comments: import.comments.iter_non_doc().cloned().collect::<Vec<_>>().join("  \n"),
        })
    }
    ctx.insert("imports", &imports);

    let content = tmpl.render("section_headers/imports.md", &ctx)?;
    let mut f = File::create(section.schema_path.join("imports.md"))?;
    write!(f, "{}", content)?;
    Ok(())
}