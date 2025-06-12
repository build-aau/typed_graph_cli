use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::io::Write;
use mdbook::{Config, MDBook};
use tera::{Context, Tera};

use crate::{GenResult, Project};
use super::{DocBookContext, populate_schema_context};

const BOOK: &'static [u8] = include_bytes!("../../book_template/book.toml");

const PRIMTIVES: &'static [u8] = include_bytes!("../../book_template/src/primitives.md");

const TEMPLATES: &'static [(&'static str, &'static str)] = &[
    
    ("macros/fields.tera", include_str!("../../book_template/templates/macros/fields.tera")),

    ("src/SUMMARY.md", include_str!("../../book_template/src/SUMMARY.md")),

    ("section_headers/schema.md", include_str!("../../book_template/templates/section_headers/schema.md")),
    ("section_headers/edges.md", include_str!("../../book_template/templates/section_headers/edges.md")),
    ("section_headers/imports.md", include_str!("../../book_template/templates/section_headers/imports.md")),
    ("section_headers/nodes.md", include_str!("../../book_template/templates/section_headers/nodes.md")),
    ("section_headers/types.md", include_str!("../../book_template/templates/section_headers/types.md")),
    ("section_headers/structs.md", include_str!("../../book_template/templates/section_headers/structs.md")),
    
    ("statements/edge.md", include_str!("../../book_template/templates/statements/edge.md")),
    ("statements/node.md", include_str!("../../book_template/templates/statements/node.md")),
    ("statements/type.md", include_str!("../../book_template/templates/statements/type.md")),
    ("statements/struct.md", include_str!("../../book_template/templates/statements/struct.md")),
];

macro_rules! write_file {
    ($p:expr, $d:expr) => {
        let mut f = File::create($p)?;
        f.write_all($d)?;
    };
}

pub fn create_doc_book(prj: &Project, out_dir: impl AsRef<Path>) -> GenResult<()> {
    // let tmp = TempDir::new("typed_graph")?;
    //let tmp_path = tmp.path();
    let out_dir = out_dir.as_ref();
    if !out_dir.exists() {
        create_dir_all(out_dir)?;
    }
    let out_src_dir = out_dir.join("src");
    if !out_src_dir.exists() {
        create_dir_all(&out_src_dir)?;
    }

    let out_res_dir = out_src_dir.join("resources");
    if !out_res_dir.exists() {
        create_dir_all(&out_res_dir)?;
    }

    copy_resources(out_dir)?;

    let mut book_gen = DocBookContext::new(&out_dir)?;
    let mut ctx = Context::new();

    let tmpl = build_tera()?;

    populate_schema_context(&mut book_gen, &tmpl, prj)?;

    book_gen.add_to_context(&mut ctx);
    
    let summary_path = out_src_dir.join("SUMMARY.md");
    let summary_s = tmpl.render("src/SUMMARY.md", &ctx)?;
    write_file!(summary_path, summary_s.as_bytes());
    Ok(())
}

pub fn build_doc_book(out_dir: impl AsRef<Path>) -> GenResult<()> {
    MDBook::load(out_dir.as_ref())?.build()?;
    Ok(())
}

pub fn build_doc_book_with_target(out_dir: impl AsRef<Path>, target: impl AsRef<Path>) -> GenResult<()> {
    let current = current_dir()?;
    let mut config = Config::from_disk(out_dir.as_ref().join("book.toml"))?;
    config.build.build_dir = current.join(target);
    MDBook::load_with_config(out_dir.as_ref(), config)?.build()?;
    Ok(())
}

pub(super) fn copy_resources(path: impl AsRef<Path>) -> GenResult<()> {
    let root_path = path.as_ref();
    write_file!(root_path.join("book.toml"), BOOK);
    let src_path = root_path.join("src");
    write_file!(src_path.join("primitives.md"), PRIMTIVES);
    Ok(())
}

fn build_tera() -> GenResult<Tera> {
    let mut tmpl = Tera::default();
    tmpl.add_raw_templates(TEMPLATES.into_iter().cloned())?;

    tmpl.set_escape_fn(|s| s.to_string());
    Ok(tmpl)
}

#[test]
fn aa() -> GenResult<()> {
    let prj = Project::open_project("project")?;
    create_doc_book(&prj, "test_dir")?;
    build_doc_book("test_dir")?;
    Ok(())
}