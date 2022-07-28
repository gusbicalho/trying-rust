#![allow(dead_code)]

use std::marker::PhantomData;

use derivative::Derivative;

use super::internals::{Parser, ParserState};

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Cloning<R, E = !> {
    val: R,
    _phantom: PhantomData<E>,
}

pub fn cloning<R, E>(val: R) -> Cloning<R, E>
where
    R: Clone,
{
    Cloning {
        val,
        _phantom: PhantomData,
    }
}

impl<R, E> Parser for Cloning<R, E>
where
    R: Clone,
{
    type Item = R;
    type ParseError = E;
    fn parse(&self, _: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        Ok(self.val.clone())
    }
}

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Lazy<F> {
    make_result: F,
}

pub fn lazy<F>(make_result: F) -> Lazy<F> {
    Lazy { make_result }
}

impl<F, R, E> Parser for Lazy<F>
where
    F: Fn() -> Result<R, E>,
{
    type Item = R;
    type ParseError = E;
    fn parse(&self, _: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        (self.make_result)()
    }
}
