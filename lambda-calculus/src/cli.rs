mod config;
use std::error::Error;

pub use config::{print_usage, Config};

use crate::parser::{self, Parser};

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn run_stmt(line: String) -> Result<(), Box<dyn Error>> {
    let parser = parser::parse_stmt();
    match parser.parse_str(&line) {
        Err(parse_err) => Err(parse_err)?,
        Ok(stmt) => {
            println!("{:#?}", stmt);
            &parser;
            Ok(())
        }
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if let Err(err) = run_stmt(line) {
                    eprintln!("Parse error: {}", err)
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                eprintln!("Input error: {:?}", err);
                break
            }
        }
    }


    // let mut stdin = io::stdin().lock();
    // let mut stmt_buf = String::new();
    // let mut line_buf = String::new();
    // stdin
    //     .lines()
    //     .scan(String::new(), |stmt_buf, line| {
    //         match line {
    //             Err(err) => Some(Err(err)),
    //             Ok(line) => {
    //                 let trimmed = line.trim();
    //                 if trimmed.is_empty() {
    //                     let stmt = stmt_buf.clone();
    //                     stmt_buf.clear();
    //                     Some(Ok(stmt))
    //                 } else {
    //                     stmt_buf.push_str(trimmed);
    //                     None
    //                 }
    //             },
    //         }
    //         // line.map(|line| {
    //         //     let trimmed = line.trim();
    //         // })

    //     });
    // while 0 != stdin.read_line(&mut line_buf)? {
    //     let trimmed = line_buf.trim();
    //     if trimmed.is_empty() {

    //     } else {
    //         stmt_buf.pu
    //     }
    // }
    // let input: String = {
    //     let mut buf = String::new();
    //     io::stdin()
    //         .lock()
    //         .read_to_string(&mut buf)
    //         .expect("Failure to read from stdin");
    //     buf
    // };
    // let stmts = parser::parse_stmts(input);
    // for stmt in stmts {
    //     println!("{:#?}", stmts);
    // }
    // if config.is_interactive() {

    // }
    Ok(())
}
