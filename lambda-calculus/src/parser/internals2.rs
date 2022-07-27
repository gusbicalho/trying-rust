use std::{error::Error, marker::PhantomData};

use derivative::Derivative;

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

    fn advance(&mut self, distance: usize) {
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

    fn current_position(&self) -> ParserPos {
        self.position.clone()
    }
}

fn test() -> Result<String, String> {
    let p = pure::<_, ()>(42);
    p.map(|i| i.to_string())
        .map_err(|_| "wat".to_string())
        .clone()
        .parse(&mut ParserState::new(""))
}

pub trait Parser<'a> {
    type Item;
    type ParseError = Box<dyn Error>;
    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError>;

    fn map<I2, F: Fn(Self::Item) -> I2>(self, transform: F) -> Map<Self, F>
    where
        Self: std::marker::Sized,
    {
        Map::new(self, transform)
    }

    fn map_err<E2, F: Fn(Self::ParseError) -> E2>(self, transform: F) -> MapErr<Self, F>
    where
        Self: std::marker::Sized,
        MapErr<Self, F> : Parser<'a, Item = Self::Item, ParseError = E2>
    {
        MapErr::new(self, transform)
    }
}

impl<'a, T> Parser<'a> for &T
where
    T: Parser<'a>,
{
    type Item = T::Item;

    type ParseError = T::ParseError;

    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        (*self).parse(state)
    }
}

// Pure

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Pure<R, E = !> {
    val: R,
    _phantom: PhantomData<E>,
}

impl<'a, R, E> Parser<'a> for Pure<R, E>
where
    R: 'a,
{
    type Item = &'a R;
    type ParseError = E;
    fn parse(&'a self, state: &mut ParserState) -> Result<&'a R, E> {
        Ok(&self.val)
    }
}

fn pure<R, E>(val: R) -> Pure<R, E> {
    Pure {
        val,
        _phantom: PhantomData,
    }
}

// Map

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct Map<P, F> {
    parser: P,
    transform: F,
}

impl<P, F> Map<P, F> {
    pub fn new(parser: P, transform: F) -> Self {
        Self { parser, transform }
    }
}

impl<'a, P, F, I2> Parser<'a> for Map<P, F>
where
    P: Parser<'a>,
    F: Fn(P::Item) -> I2,
{
    type Item = I2;

    type ParseError = P::ParseError;

    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        self.parser.parse(state).map(self.transform)
    }
}

// MapErr

#[derive(Derivative)]
#[derivative(Copy, Clone)]
pub struct MapErr<P, F> {
    parser: P,
    transform: F,
}

impl<P, F> MapErr<P, F> {
    pub fn new(parser: P, transform: F) -> Self {
        Self { parser, transform }
    }
}

impl<'a, P, F, E2> Parser<'a> for MapErr<P, F>
where
    P: Parser<'a>,
    F: Fn(P::ParseError) -> E2,
{
    type Item = P::Item;

    type ParseError = E2;

    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        self.parser.parse(state).map_err(self.transform)
    }
}

// pub struct Parser<P, R, E = Box<dyn Error>>
// where
//     P: IsParserFn<R, E>,
// {
//     parser_fn: P,
//     q: PhantomData<(R, E)>,
// }

// impl<P, R, E> Parser<P, R, E>
// where
//     P: IsParserFn<R, E>,
// {
//     pub fn new(run: P) -> Self {
//         Self {
//             parser_fn: run,
//             q: PhantomData,
//         }
//     }

//     pub fn parse(&self, state: &mut ParserState) -> Result<R, E> {
//         (self.parser_fn)(state)
//     }

//     // Parser transformers

//     pub fn map<'a, S>(
//         &'a self,
//         f: impl 'a + Fn(R) -> S,
//     ) -> Parser<impl IsParserFn<S, E> + 'a, S, E> {
//         Parser::new(move |state: &mut ParserState| match self.parse(state) {
//             Ok(r) => Ok(f(r)),
//             Err(err) => Err(err),
//         })
//     }

//     pub fn map_err<'a, E2>(
//         &'a self,
//         f: impl 'a + Fn(E) -> E2,
//     ) -> Parser<impl IsParserFn<R, E2> + 'a, R, E2> {
//         Parser::new(move |state: &mut ParserState| match self.parse(state) {
//             Ok(r) => Ok(r),
//             Err(err) => Err(f(err)),
//         })
//     }

//     pub fn map_err_into<'a, E2>(&'a self) -> Parser<impl IsParserFn<R, E2> + 'a, R, E2>
//     where
//         E2: From<E> + 'a,
//     {
//         self.map_err(|err| err.into())
//     }

//     pub fn backtracking(&self) -> Parser<impl IsParserFn<R, E> + '_, R, E> {
//         Parser::new(move |state: &mut ParserState| {
//             let backup = state.clone();
//             match self.parse(state) {
//                 Ok(r) => Ok(r),
//                 Err(err) => {
//                     *state = backup;
//                     Err(err)
//                 }
//             }
//         })
//     }

//     pub fn looking_ahead(&self) -> Parser<impl IsParserFn<R, E> + '_, R, E> {
//         Parser::new(move |state: &mut ParserState| self.parse(&mut state.clone()))
//     }

//     pub fn falling_back<'a>(
//         &'a self,
//         fallback_parser: &'a Parser<impl IsParserFn<R, E> + 'a, R, E>,
//     ) -> Parser<impl IsParserFn<R, E> + 'a, R, E> {
//         Parser::new(move |state: &mut ParserState| {
//             let previously_consumed = state.consumed_so_far;
//             match self.parse(state) {
//                 Ok(result) => Ok(result),
//                 Err(err) => {
//                     if state.consumed_so_far == previously_consumed {
//                         // main_parser did not consume any data, so we fallback
//                         fallback_parser.parse(state)
//                     } else {
//                         Err(err)
//                     }
//                 }
//             }
//         })
//     }

//     pub fn followed_by<'a, R2>(
//         &'a self,
//         next_parser: &'a Parser<impl IsParserFn<R2, E> + 'a, R2, E>,
//     ) -> Parser<impl IsParserFn<R2, E> + 'a, R2, E> {
//         Parser::new(move |state: &mut ParserState| {
//             self.parse(state)?;
//             next_parser.parse(state)
//         })
//     }

//     pub fn and_then<'a, NP, R2>(
//         &'a self,
//         make_next_parser: impl Fn(R) -> Parser<NP, R2, E> + 'a,
//     ) -> Parser<impl IsParserFn<R2, E> + 'a, R2, E>
//     where
//         NP: IsParserFn<R2, E> + 'a,
//     {
//         Parser::new(move |state: &mut ParserState| {
//             let parsed = self.parse(state)?;
//             make_next_parser(parsed).parse(state)
//         })
//     }

//     pub fn with_span(
//         &self,
//     ) -> Parser<impl IsParserFn<(R, ParserSpan), E> + '_, (R, ParserSpan), E> {
//         Parser::new(move |state: &mut ParserState| {
//             let start = state.current_position();
//             self.parse(state).map(|result| {
//                 let end = state.current_position();
//                 (result, (start, end))
//             })
//         })
//     }

//     pub fn many<E2>(&self) -> Parser<impl IsParserFn<Vec<R>, E2> + '_, Vec<R>, E2> {
//         let parse_next = self.backtracking();
//         Parser::new(move |state: &mut ParserState| {
//             let mut results: Vec<R> = vec![];
//             while let Ok(item) = parse_next.parse(state) {
//                 results.push(item);
//             }
//             Ok(results)
//         })
//     }

//     pub fn skip_many<E2>(&self) -> Parser<impl IsParserFn<(), E2> + '_, (), E2> {
//         let parse_next = self.backtracking();
//         Parser::new(move |state: &mut ParserState| {
//             while let Ok(item) = parse_next.parse(state) {}
//             Ok(())
//         })
//     }

//     pub fn skip_at_least_one(&self) -> Parser<impl IsParserFn<(), E> + '_, (), E> {
//         let parse_more = self.skip_many::<E>();
//         Parser::new(move |state: &mut ParserState| match self.parse(state) {
//             Err(err) => Err(err),
//             Ok(_) => parse_more.parse(state),
//         })
//     }
// }

// // basic parsers

// pub fn pure<R: Copy, E>(val: R) -> Parser<impl IsParserFn<R, E>, R, E> {
//     Parser {
//         parser_fn: move |_: &mut ParserState| Ok(val),
//         q: PhantomData,
//     }
// }

// pub fn one_char<'a>(
//     predicate: impl 'a + Fn(char) -> bool,
//     description: &'a str,
// ) -> Parser<impl IsParserFn<char> + 'a, char> {
//     Parser::new(
//         move |state: &mut ParserState| match state.leftovers.chars().next() {
//             Some(c) => {
//                 if predicate(c) {
//                     state.advance(c.len_utf8());
//                     Ok(c)
//                 } else {
//                     Err(format!("Unexpected char {}. Expected {}", c, description))?
//                 }
//             }
//             None => Err(format!("Unexpected end of input. Expected {}", description))?,
//         },
//     )
// }

// pub fn any_char() -> Parser<impl IsParserFn<char>, char> {
//     fn parse_any_char(state: &mut ParserState) -> Result<char, Box<dyn Error>> {
//         match state.leftovers.chars().next() {
//             Some(c) => {
//                 state.advance(c.len_utf8());
//                 Ok(c)
//             }
//             None => Err("Unexpected end of input.")?,
//         }
//     }
//     Parser::new(parse_any_char)
// }

// pub fn check(expected: &str) -> Parser<impl IsParserFn<bool, !> + '_, bool, !> {
//     Parser::new(move |state: &mut ParserState| Ok(state.leftovers.starts_with(expected)))
// }

// pub fn check_one_of<'p, 'a>(
//     possibilities: &'p [&'a str],
// ) -> Parser<impl IsParserFn<Option<&'a str>, !> + 'p, Option<&'a str>, !> {
//     Parser::new(move |state: &mut ParserState| {
//         for expected in possibilities {
//             if state.leftovers.starts_with(expected) {
//                 return Ok(Some(*expected));
//             }
//         }
//         Ok(None)
//     })
// }

// pub fn expect<'a>(expected: &'a str) -> Parser<impl IsParserFn<()> + 'a, ()> {
//     let checker = check(expected);
//     Parser::new(move |state: &mut ParserState| match checker.parse(state) {
//         Ok(true) => {
//             state.advance(expected.len());
//             Ok(())
//         }
//         Ok(false) => Err(format!(
//             "Expected\n  {}\"but found\n  {}",
//             expected,
//             &state.leftovers[..expected.len()]
//         ))?,
//         Err(_) => unreachable!(),
//     })
// }

// pub fn expect_one_of<'p, 'a>(
//     possibilities: &'p [&'a str],
// ) -> Parser<impl IsParserFn<&'a str> + 'p, &'a str> {
//     let max_len = possibilities.iter().map(|s| s.len()).max().unwrap_or(0);
//     let checker = check_one_of(possibilities);
//     Parser::new(move |state: &mut ParserState| match checker.parse(state) {
//         Ok(Some(found)) => {
//             state.advance(found.len());
//             Ok(found)
//         }
//         Ok(None) => Err(format!(
//             "Expected one of\n{}but found\n  {}",
//             {
//                 let mut buf = String::new();
//                 for p in possibilities {
//                     buf.push_str("  ");
//                     buf.push_str(p);
//                     buf.push('\n');
//                 }
//                 buf
//             },
//             &state.leftovers[..max_len]
//         ))?,
//         Err(_) => unreachable!(),
//     })
// }

// pub fn expect_end() -> Parser<impl IsParserFn<()>, ()> {
//     fn parse_expect_end(state: &mut ParserState) -> Result<(), Box<dyn Error>> {
//         if state.leftovers.is_empty() {
//             Ok(())
//         } else {
//             Err(format!(
//                 "Expected end of input, but found {}",
//                 &state.leftovers[..10]
//             ))?
//         }
//     }
//     Parser::new(parse_expect_end)
// }
