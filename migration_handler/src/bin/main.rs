use migration_handler::cli::*;
use clap::Parser;
use migration_handler::GenResult;

fn main() -> GenResult<()> {
    let args = Args::parse();
    let status = args.process(&());

    if let Err(e) = &status {
        println!("{}", e);
    }
    status?;


    Ok(())
}
