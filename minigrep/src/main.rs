use minigrep::config::{self, Config};
use std::io;
use std::process;

fn main() {
    let config = Config::from_env().unwrap_or_else(|error| {
        eprintln!("{}\n", error);
        config::print_usage(&mut io::stderr()).unwrap();
        process::exit(1);
    });
    let mut stdout = io::stdout();
    minigrep::run(&config, &mut stdout).unwrap_or_else(|error| {
        eprintln!("{}\n", error);
        process::exit(1);
    });
}
