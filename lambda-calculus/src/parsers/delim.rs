#![allow(dead_code)]

use super::internals::{Parser, ParserState};
use super::one_char;

pub fn whitespace() -> one_char::OneCharMatches<fn(char) -> bool, &'static str> {
    one_char::matches(char::is_whitespace, "whitespace")
}

#[derive(Copy, Clone)]
pub struct ExpectEnd {}

pub const EXPECT_END: ExpectEnd = ExpectEnd {};

impl Parser for ExpectEnd {
    type Item = ();
    type ParseError = String;
    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        if state.leftovers().is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Expected end of input, but found {}",
                &state.leftovers()[..10.min(state.leftovers().len())]
            ))?
        }
    }
}
