use std::{process, io, error::Error};

use lambda_calculus::cli;

fn main() -> Result<(), Box<dyn Error>> {
    let config = cli::Config::from_env().unwrap_or_else(|error| {
        eprintln!("{}\n", error);
        cli::print_usage(&mut io::stderr()).unwrap();
        process::exit(1)
    });
    cli::run(&config)?;
    Ok(())
}
