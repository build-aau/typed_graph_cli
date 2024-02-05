use clap::Parser;
use typed_graph_cli::cli::*;
use typed_graph_cli::GenResult;

fn main() -> GenResult<()> {
    let args = Args::parse();
    let status = args.process(&());

    if let Err(e) = &status {
        println!("{}", e);
    }
    status?;

    Ok(())
}
