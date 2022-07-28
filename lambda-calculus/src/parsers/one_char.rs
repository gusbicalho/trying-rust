#![allow(dead_code)]

use std::fmt::Display;

use super::internals::{Parser, ParserState};

#[derive(Copy, Clone)]
pub struct OneCharMatches<Pred, Desc> {
    predicate: Pred,
    description: Desc,
}

pub fn matches<Pred, Desc>(predicate: Pred, description: Desc) -> OneCharMatches<Pred, Desc> {
    OneCharMatches {
        predicate,
        description,
    }
}

impl<Pred, Desc> Parser for OneCharMatches<Pred, Desc>
where
    Pred: Fn(char) -> bool,
    Desc: Display,
{
    type Item = char;
    type ParseError = String;
    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        match state.leftovers().chars().next() {
            Some(c) => {
                if (self.predicate)(c) {
                    state.advance(c.len_utf8());
                    Ok(c)
                } else {
                    Err(format!(
                        "Unexpected char {}. Expected {}",
                        c, self.description
                    ))?
                }
            }
            None => Err(format!(
                "Unexpected end of input. Expected {}",
                self.description
            ))?,
        }
    }
}

#[derive(Copy, Clone)]
pub struct AnyChar {}

pub const ANY: AnyChar = AnyChar {};

impl Parser for AnyChar {
    type Item = char;
    type ParseError = String;
    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        match state.leftovers().chars().next() {
            Some(c) => {
                state.advance(c.len_utf8());
                Ok(c)
            }
            None => Err("Unexpected end of input.")?,
        }
    }
}
