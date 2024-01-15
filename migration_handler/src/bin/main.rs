use typed_graph_rust_gen::cli::*;
use clap::Parser;
use typed_graph_rust_gen::GenResult;

fn main() -> GenResult<()> {
    let args = Args::parse();
    let status = args.process(&());

    if let Err(e) = &status {
        println!("{}", e);
    }
    status?;


    Ok(())
}
