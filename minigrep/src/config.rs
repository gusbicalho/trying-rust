use std::{env, io};

#[derive(Debug)]
pub struct Config {
    pub search_string: String,
    pub filename: String,
    pub case_sensitive: bool,
}

impl Config {
    pub fn from_strings(
        mut args: impl Iterator<Item = String>,
        get_env: impl Fn(&str) -> Option<String>,
    ) -> Result<Config, String> {
        let search_string = args.next().ok_or("Missing search string")?;
        let filename = args.next().ok_or("Missing filename")?;
        let case_sensitive = get_env("CASE_INSENSITIVE").is_none();
        Ok(Config {
            search_string,
            filename,
            case_sensitive,
        })
    }

    pub fn from_env() -> Result<Config, String> {
        Config::from_strings(
            // skip(1) to ignore argv[0]
            env::args().skip(1),
            |var_name| env::var(var_name).ok(),
        )
    }
}

pub fn print_usage(out: &mut impl io::Write) -> io::Result<()> {
    out.write_fmt(format_args!("minigrep <search_string> <filename>\n"))?;
    Ok(())
}
