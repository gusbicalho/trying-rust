mod config;
use std::error::Error;
use std::rc::Rc;

pub use config::{print_usage, Config};

use crate::pipeline::{evaluator, parse_to_locally_nameless, parse_tree, parser, runtime};

use rustyline::error::ReadlineError;
use rustyline::Editor;

struct Runner {
    globals: runtime::Globals,
}

impl Runner {
    fn new() -> Self {
        let mut globals = runtime::Globals::new();
        fn cast_to_integer(value: runtime::RTValue) -> Result<i64, String> {
            match value {
                runtime::RTValue::Integer(value) => Ok(value),
                other => Err(format!("Expected number, got {}", other))?,
            }
        }
        globals.define(
            "plus",
            runtime::RTValue::Function(Rc::new(|arg1, _| {
                let arg1 = cast_to_integer(arg1)?;
                Ok(runtime::RTValue::Function(Rc::new(move |arg2, _| {
                    let arg2 = cast_to_integer(arg2)?;
                    Ok(runtime::RTValue::Integer(arg1 + arg2))
                })))
            })),
        );
        globals.define(
            "repeatedly",
            runtime::RTValue::Function(Rc::new(|number_of_times, _| {
                let number_of_times = cast_to_integer(number_of_times)?;
                Ok(runtime::RTValue::Function(Rc::new(move |function, _| {
                    let function = match function {
                        runtime::RTValue::Function(f) => f,
                        other => Err(format!("Expected function, got {}", other))?,
                    };
                    Ok(runtime::RTValue::Function(Rc::new(move |seed, globals| {
                        let mut v = seed;
                        for _ in 0..number_of_times {
                            v = (*function)(v, globals)?;
                        }
                        Ok(v)
                    })))
                })))
            })),
        );
        Self { globals }
    }

    fn run_stmt(&mut self, line: String) -> Result<(), Box<dyn Error>> {
        let parsed_stmt = parser::parse_stmt(&line)?;
        let (identifier, expr) = match parsed_stmt {
            parse_tree::Stmt::Decl(parse_tree::Decl { identifier, expr }) => {
                (Some(identifier), expr)
            }
            parse_tree::Stmt::Expr(expr) => (None, expr),
        };
        let runnable_expr = parse_to_locally_nameless::transform_expr(&expr);
        let value = evaluator::eval(&runnable_expr, &self.globals)?;
        if let Some(identifier) = identifier {
            self.globals.define(&identifier[..], value);
        } else {
            println!("{}", value);
        }
        Ok(())
    }
}

pub fn run(_config: &Config) -> Result<(), Box<dyn Error>> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history(".lambda_calculus_history").is_err() {}
    let mut runner = Runner::new();
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
