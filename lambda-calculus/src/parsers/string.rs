#![allow(dead_code)]

use std::marker::PhantomData;

use derivative::Derivative;

use super::internals::{Parser, ParserState};

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Check<'a, E = !> {
    expected: &'a str,
    phantom: PhantomData<E>,
}

pub fn check<E>(expected: &str) -> Check<E> {
    Check {
        expected,
        phantom: PhantomData,
    }
}
impl<'a, E> Parser for Check<'a, E> {
    type Item = bool;

    type ParseError = E;

    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        Ok(state.leftovers().starts_with(self.expected))
    }
}

#[derive(Derivative)]
#[derivative(Clone)]
pub struct CheckOwned<E = !> {
    expected: String,
    phantom: PhantomData<E>,
}

pub fn check_owned<E>(expected: String) -> CheckOwned<E> {
    CheckOwned {
        expected,
        phantom: PhantomData,
    }
}
impl<E> Parser for CheckOwned<E> {
    type Item = bool;

    type ParseError = E;

    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        check(&self.expected).parse(state)
    }
}

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Expect<'a> {
    expected: &'a str,
}

pub fn expect(expected: &str) -> Expect {
    Expect { expected }
}
impl<'a> Parser for Expect<'a> {
    type Item = ();

    type ParseError = String;

    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        match check::<!>(self.expected).parse(state) {
            Ok(true) => {
                state.advance(self.expected.len());
                Ok(())
            }
            Ok(false) => Err(format!(
                "Expected\n  {}\nbut found\n  {}",
                self.expected,
                &state.leftovers()[..self.expected.len().min(state.leftovers().len())]
            )),
            Err(_) => unreachable!(),
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone)]
pub struct ExpectOwned {
    expected: String,
}

pub fn expect_owned(expected: String) -> ExpectOwned {
    ExpectOwned { expected }
}
impl Parser for ExpectOwned {
    type Item = ();

    type ParseError = String;

    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        expect(&self.expected).parse(state)
    }
}

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct ManyCharsMatching<Pred, E = !> {
    predicate: Pred,
    phantom: PhantomData<E>,
}

pub fn many_chars_matching<Pred, E>(predicate: Pred) -> ManyCharsMatching<Pred, E> {
    ManyCharsMatching {
        predicate,
        phantom: PhantomData,
    }
}

impl<Pred, E> Parser for ManyCharsMatching<Pred, E>
where
    Pred: Fn(char) -> bool,
{
    type Item = String;
    type ParseError = E;
    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        Ok({
            let mut s = String::new();
            s.extend(state.leftovers().chars().take_while(|c| (self.predicate)(*c)));
            state.advance(s.len());
            s
        })
    }
}
