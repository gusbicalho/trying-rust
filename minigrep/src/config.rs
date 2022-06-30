use std::{env, io};

#[derive(Debug)]
pub struct Config {
    pub search_string: String,
    pub filename: String,
    pub case_sensitive: bool,
}

impl Config {
    pub fn from_strings(args: &mut impl Iterator<Item = String>) -> Result<Config, String> {
        let search_string = args.next().ok_or("Missing search string")?;
        let filename = args.next().ok_or("Missing filename")?;
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();
        Ok(Config {
            search_string,
            filename,
            case_sensitive,
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
