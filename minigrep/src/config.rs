use std::{env, io};

#[derive(Debug)]
pub struct Config {
    pub search_string: String,
    pub filename: String,
}

impl Config {
    pub fn from_strings(args: &mut impl Iterator<Item = String>) -> Result<Config, String> {
        let search_string = args.next().ok_or("Missing search string")?;
        let filename = args.next().ok_or("Missing filename")?;
        Ok(Config {
            search_string,
            filename,
        })
    }

    pub fn from_env() -> Result<Config, String> {
        let mut args = env::args();
        // skip argv[0]
        args.next().ok_or("empty argv")?;
        Config::from_strings(&mut args)
    }
}

pub fn print_usage(out: &mut impl io::Write) -> io::Result<()> {
    out.write_fmt(format_args!("minigrep <search_string> <filename>\n"))?;
    Ok(())
}
