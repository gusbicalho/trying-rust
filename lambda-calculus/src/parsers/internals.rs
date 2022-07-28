#![allow(dead_code)]

use std::{error::Error, ops::Deref};

use super::adapters::{
    also::Also, at_least_one::AtLeastOne, backtracking::Backtracking, falling_back::FallingBack,
    looking_ahead::LookingAhead, many::Many, map::Map, map_err::MapErr, optional::Optional,
    paired_with::PairedWith, skip_many::SkipMany, then::Then, validate::Validate,
    with_span::WithSpan,
};

#[derive(Clone)]
pub struct ParserPos {
    pub line: usize,
    pub column: usize,
}

impl ParserPos {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

pub type ParserSpan = (ParserPos, ParserPos);

#[derive(Clone)]
pub struct ParserState<'a> {
    leftovers: &'a str,
    consumed_so_far: usize,
    position: ParserPos,
}

impl<'a> ParserState<'a> {
    pub fn new(text: &str) -> ParserState {
        ParserState {
            leftovers: text,
            consumed_so_far: 0,
            position: ParserPos::new(0, 0),
        }
    }

    pub fn advance(&mut self, distance: usize) {
        let (consumed, leftovers) = self.leftovers.split_at(distance);
        self.leftovers = leftovers;
        self.consumed_so_far += consumed.len();
        let mut newlines = consumed.rmatch_indices('\n');
        match newlines.next() {
            None => self.position.column += consumed.len(),
            Some((index_of_last_newline, _)) => {
                self.position.column = consumed.len() - index_of_last_newline - 1;
                self.position.line += newlines.count() + 1;
            }
        }
    }

    pub fn leftovers(&self) -> &'a str {
        self.leftovers
    }

    pub fn current_position(&self) -> &ParserPos {
        &self.position
    }

    pub fn consumed_so_far(&self) -> usize {
        self.consumed_so_far
    }
}

pub trait Parser {
    type Item;
    type ParseError = Box<dyn Error>;
    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError>;

    fn parse_str(&self, text: &str) -> Result<Self::Item, Self::ParseError> {
        self.parse(&mut ParserState::new(text))
    }

    fn map<I2, F: Fn(Self::Item) -> I2>(self, transform: F) -> Map<Self, F>
    where
        Self: std::marker::Sized,
    {
        Map::new(self, transform)
    }

    fn map_err<E2, F: Fn(Self::ParseError) -> E2>(self, transform: F) -> MapErr<Self, F>
    where
        Self: std::marker::Sized,
        MapErr<Self, F>: Parser<ParseError = E2>,
    {
        MapErr::new(self, transform)
    }

    fn validate<F>(self, validate: F) -> Validate<Self, F>
    where
        Self: std::marker::Sized,
        Validate<Self, F>: Parser,
    {
        Validate::new(self, validate)
    }

    fn map_err_into<E2>(self) -> MapErr<Self, fn(Self::ParseError) -> E2>
    where
        Self: std::marker::Sized,
        E2: From<Self::ParseError>,
    {
        self.map_err(Into::into)
    }

    fn optional(self) -> Optional<Self>
    where
        Self: std::marker::Sized,
    {
        Optional::new(self)
    }

    fn backtracking(self) -> Backtracking<Self>
    where
        Self: std::marker::Sized,
    {
        Backtracking::new(self)
    }

    fn looking_ahead(self) -> LookingAhead<Self>
    where
        Self: std::marker::Sized,
    {
        LookingAhead::new(self)
    }

    fn falling_back<P2>(self, fallback_parser: P2) -> FallingBack<Self, P2>
    where
        Self: std::marker::Sized,
        FallingBack<Self, P2>: Parser,
    {
        FallingBack::new(self, fallback_parser)
    }

    fn then<P2>(self, next_parser: P2) -> Then<Self, P2>
    where
        Self: std::marker::Sized,
        Then<Self, P2>: Parser,
    {
        Then::new(self, next_parser)
    }

    fn also<P2>(self, next_parser: P2) -> Also<Self, P2>
    where
        Self: std::marker::Sized,
        Also<Self, P2>: Parser,
    {
        Also::new(self, next_parser)
    }

    fn paired_with<P2>(self, next_parser: P2) -> PairedWith<Self, P2>
    where
        Self: std::marker::Sized,
        PairedWith<Self, P2>: Parser,
    {
        PairedWith::new(self, next_parser)
    }

    fn many<E>(self) -> Many<Self, E>
    where
        Self: std::marker::Sized,
    {
        Many::new(self)
    }

    fn skip_many<E>(self) -> SkipMany<Self, E>
    where
        Self: std::marker::Sized,
    {
        SkipMany::new(self)
    }

    fn at_least_one(self) -> AtLeastOne<Self>
    where
        Self: std::marker::Sized + Clone,
    {
        AtLeastOne::new(self)
    }

    fn skip_at_least_one(self) -> Then<Self, SkipMany<Self, Self::ParseError>>
    where
        Self: std::marker::Sized + Clone,
    {
        self.clone().then(self.skip_many::<Self::ParseError>())
    }

    fn with_span(self) -> WithSpan<Self>
    where
        Self: std::marker::Sized,
    {
        WithSpan::new(self)
    }
}

impl<T> Parser for T
where
    T: Deref,
    <T as Deref>::Target: Parser,
{
    type Item = <<T as Deref>::Target as Parser>::Item;

    type ParseError = <<T as Deref>::Target as Parser>::ParseError;

    fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        (**self).parse(state)
    }
}
