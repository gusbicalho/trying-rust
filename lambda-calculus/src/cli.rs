mod config;
use std::error::Error;

pub use config::{print_usage, Config};

use crate::pipeline::parser;

use rustyline::error::ReadlineError;
use rustyline::Editor;

struct Runner {}

impl Runner {
    fn new() -> Self {
        Self {}
    }

    fn run_stmt(&self, line: String) -> Result<(), Box<dyn Error>> {
        match parser::parse_stmt(&line) {
            Err(parse_err) => Err(parse_err)?,
            Ok(stmt) => {
                println!("{:#?}", stmt);
                Ok(())
            }
        }
    }
}

pub fn run(_config: &Config) -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history(".lambda_calculus_history").is_err() {}
    let runner = Runner::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if let Err(err) = runner.run_stmt(line) {
                    eprintln!("Parse error: {}", err)
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Input error: {:?}", err);
                break;
            }
        }
    }
    if rl.save_history(".lambda_calculus_history").is_err() {};
    Ok(())
}
