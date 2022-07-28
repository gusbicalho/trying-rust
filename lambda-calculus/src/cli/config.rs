use std::{env, io};

#[derive(Debug)]
pub struct Config {}

impl Config {
    pub fn from_strings(_arg_strs: impl Iterator<Item = String>) -> Result<Config, String> {
        let result = Config {};
        Ok(result)
    }

    pub fn from_env() -> Result<Config, String> {
        Config::from_strings(
            // skip(1) to ignore argv[0]
            env::args().skip(1),
        )
    }
}

pub fn print_usage(out: &mut impl io::Write) -> io::Result<()> {
    out.write_fmt(format_args!("lambda-calculus\n"))?;
    Ok(())
}
