pub mod config;

use std::{error::Error, fs::File, io::Write};

pub fn run(config: &config::Config, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let file = File::open(&config.filename)?;
    query::search(&config.search_string, file, &mut |line| {
        output.write_fmt(format_args!("{}\n", line))
    })?;
    Ok(())
}

mod query {
    use std::io::{self, BufRead, BufReader, Read};

    pub fn search<Out: FnMut(&str) -> io::Result<()>>(
        query: &str,
        input: impl Read,
        output: &mut Out,
    ) -> io::Result<()> {
        for line in BufReader::new(input).lines() {
            let line = line?;
            if let Some(result) = matches(&line, query) {
                output(result)?;
            }
        }
        Ok(())
    }

    pub fn matches<'s>(s: &'s str, query: &str) -> Option<&'s str> {
        if s.contains(query) {
            Some(s)
        } else {
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn one_result() {
            let query = "duct";
            let contents = "\
Rust:
safe, fast, productive.
Pick three.";
            assert_eq!(
                vec!["safe, fast, productive."],
                search_on_string(query, contents)
            );
        }

        fn search_on_string(query: &str, contents: &str) -> Vec<String> {
            let mut out: Vec<String> = Vec::new();
            search(query, contents.as_bytes(), &mut |line| {
                out.push(line.to_string());
                Ok(())
            })
            .unwrap();
            out
        }
    }
}
