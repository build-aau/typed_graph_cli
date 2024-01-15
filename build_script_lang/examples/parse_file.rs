use build_script_lang::{DefaultSchema, BUILDScriptResult};
use build_script_shared::parsers::ParserDeserialize;

fn main() -> BUILDScriptResult<()> {
    use std::fs::read_to_string;
    use std::env::var;
    use std::path::Path;
    let p = Path::new(&var("CARGO_MANIFEST_DIR").unwrap()).join("examples/test_files/simple.bs");
    let s = read_to_string(p)?;
    let res = DefaultSchema::deserialize(&s);
    match &res {
        Ok(v) => {
            dbg!(v);
        }
        Err(e) => println!("{}", e),
    };
    Ok(())
}