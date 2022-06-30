#![feature(trait_alias)]

pub mod config;

use std::{error::Error, fs::File, io::Write};

use query::matcher::Matcher;

pub fn run(config: &config::Config, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let file = File::open(&config.filename)?;
    query::search(
        if config.case_sensitive {
            Box::new(query::matcher::matches_case_sensitive(
                &config.search_string,
            )) as Box<dyn Matcher>
        } else {
            Box::new(query::matcher::matches_case_insensitive(
                &config.search_string,
            )) as Box<dyn Matcher>
        },
        file,
        &mut |line| output.write_fmt(format_args!("{}\n", line)),
    )?;
    Ok(())
}

mod query {
    use std::io::{self, BufRead, BufReader, Read};

    pub fn search<Out: FnMut(&str) -> io::Result<()>>(
        matches: impl matcher::Matcher,
        input: impl Read,
        output: &mut Out,
    ) -> io::Result<()> {
        for line in BufReader::new(input).lines() {
            let line = line?;
            if let Some(result) = matches(&line) {
                output(result)?;
            }
        }
        Ok(())
    }

    pub mod matcher {
        pub trait Matcher = for<'a> Fn(&'a str) -> Option<&'a str>;

        pub fn matches_case_sensitive(
            query: &str,
        ) -> impl for<'a> Fn(&'a str) -> Option<&'a str> + '_ {
            move |s| {
                if s.contains(query) {
                    Some(s)
                } else {
                    None
                }
            }
        }

        pub fn matches_case_insensitive(
            query: &str,
        ) -> impl for<'a> Fn(&'a str) -> Option<&'a str> + '_ {
            let query = query.to_lowercase();
            move |s| {
                if s.to_lowercase().contains(&query) {
                    Some(s)
                } else {
                    None
                }
            }
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
            let mut out = Vec::new();
            search(
                matcher::matches_case_sensitive(query),
                contents.as_bytes(),
                &mut |line| {
                    out.push(line.to_string());
                    Ok(())
                },
            )
            .unwrap();
            out
        }
    }
}
