use std::fs::create_dir_all;

use build_script_lang::schema::Schema;
use build_script_shared::InputMarker;
use tera::{Context, Tera};

use crate::book_generation::DocBookContext;
use crate::cli::export_svg;
use crate::{GenResult, Project};
use super::*;

pub fn populate_schema_context(book_gen: &mut DocBookContext, tmpl: &Tera, prj: &Project) -> GenResult<()> {
    
    let heads = prj.find_heads();

    for schema_id in &heads {
        println!("Creating schema {}", schema_id);
        let schema = prj.get_schema(schema_id)?;
        let schema_content = populate_schema(schema, book_gen, tmpl)?;
        let mut section = book_gen.create_main_section(schema.version.to_string(), schema_content)?;
        populate_section_header(&mut section, schema, tmpl)?;
        populate_statement_context(&mut section, schema, tmpl)?;
    }

    for schema_id in prj.iter_schema() {
        if heads.contains(schema_id) {
            continue;
        }

        println!("Creating schema {}", schema_id);

        let schema = prj.get_schema(schema_id)?;
        let schema_content = populate_schema(schema, book_gen, tmpl)?;
        let mut section = book_gen.create_other_section(schema.version.to_string(), schema_content)?;
        populate_section_header(&mut section, schema, tmpl)?;
        populate_statement_context(&mut section, schema, tmpl)?;
    }

    Ok(())
}


pub fn populate_schema(schema: &Schema<InputMarker<String>>, book_gen: &DocBookContext, tmpl: &Tera) -> GenResult<String>  {
    
    let svg_path = book_gen.book_res_path.join("diagrams");
    if !svg_path.exists() {
        create_dir_all(&svg_path)?;
    }
    let abs_out_path = export_svg(schema, &svg_path)?;
    let out_path = abs_out_path.strip_prefix(&book_gen.book_src_path)?;

    let mut ctx = Context::new();

    ctx.insert("svg_path", &out_path);
    
    Ok(tmpl.render("section_headers/schema.md", &ctx)?)
}