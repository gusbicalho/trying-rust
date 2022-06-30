use minigrep::config::{self, Config};
use std::io::{self, Write};
use std::process;

fn main() {
    let config = Config::from_env().unwrap_or_else(|error| {
        let mut stderr = io::stderr().lock();

        stderr.write_fmt(format_args!("{}\n", error)).unwrap();
        config::print_usage(&mut io::stderr()).unwrap();
        process::exit(1);
    });
    let mut stdout = io::stdout();
    minigrep::run(&config, &mut stdout).unwrap_or_else(|error| {
        let mut stderr = io::stderr().lock();
        stderr.write_fmt(format_args!("{}\n", error)).unwrap();
        process::exit(1);
    });
}
