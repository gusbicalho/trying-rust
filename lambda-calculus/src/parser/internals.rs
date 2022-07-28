use std::{error::Error, ops::Deref};

use self::adapters::{
    also::Also, at_least_one::AtLeastOne, backtracking::Backtracking, bind::Bind,
    falling_back::FallingBack, looking_ahead::LookingAhead, many::Many, map::Map, map_err::MapErr,
    optional::Optional, paired_with::PairedWith, skip_many::SkipMany, then::Then,
    validate::Validate, with_span::WithSpan,
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

pub trait Parser<'a> {
    type Item;
    type ParseError = Box<dyn Error>;
    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError>;

    fn parse_str(&'a self, text: &'a str) -> Result<Self::Item, Self::ParseError> {
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
        MapErr<Self, F>: Parser<'a, ParseError = E2>,
    {
        MapErr::new(self, transform)
    }

    fn validate<F>(self, validate: F) -> Validate<Self, F>
    where
        Self: std::marker::Sized,
        Validate<Self, F>: Parser<'a>,
    {
        Validate::new(self, validate)
    }

    fn map_err_into<E2>(self) -> MapErr<Self, fn(Self::ParseError) -> E2>
    where
        Self: std::marker::Sized,
        E2: From<Self::ParseError> + 'a,
    {
        self.map_err(Into::into)
    }

    fn optional<E>(self) -> Optional<Self, E>
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
        FallingBack<Self, P2>: Parser<'a>,
    {
        FallingBack::new(self, fallback_parser)
    }

    fn then<P2>(self, next_parser: P2) -> Then<Self, P2>
    where
        Self: std::marker::Sized,
        Then<Self, P2>: Parser<'a>,
    {
        Then::new(self, next_parser)
    }

    fn also<P2>(self, next_parser: P2) -> Also<Self, P2>
    where
        Self: std::marker::Sized,
        Also<Self, P2>: Parser<'a>,
    {
        Also::new(self, next_parser)
    }

    fn paired_with<P2>(self, next_parser: P2) -> PairedWith<Self, P2>
    where
        Self: std::marker::Sized,
        PairedWith<Self, P2>: Parser<'a>,
    {
        PairedWith::new(self, next_parser)
    }

    fn bind<F>(self, make_next_parser: F) -> Bind<Self, F>
    where
        Self: std::marker::Sized,
        Bind<Self, F>: Parser<'a>,
    {
        Bind::new(self, make_next_parser)
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

impl<'a, T> Parser<'a> for T
where
    T: Deref,
    <T as Deref>::Target: Parser<'a>,
{
    type Item = <<T as Deref>::Target as Parser<'a>>::Item;

    type ParseError = <<T as Deref>::Target as Parser<'a>>::ParseError;

    fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
        (**self).parse(state)
    }
}

pub mod adapters {
    pub mod map {
        use derivative::Derivative;

        use super::super::{Parser, ParserState};

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
                self.parser.parse(state).map(&self.transform)
            }
        }
    }

    pub mod validate {
        use derivative::Derivative;

        use super::super::{Parser, ParserState};

        #[derive(Derivative)]
        #[derivative(Copy, Clone)]
        pub struct Validate<P, F> {
            parser: P,
            validate: F,
        }

        impl<P, F> Validate<P, F> {
            pub fn new(parser: P, validate: F) -> Self {
                Self { parser, validate }
            }
        }

        impl<'a, P, F> Parser<'a> for Validate<P, F>
        where
            P: Parser<'a>,
            F: Fn(&P::Item) -> Option<P::ParseError>,
        {
            type Item = P::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                match self.parser.parse(state) {
                    Err(err) => Err(err),
                    Ok(val) => match (self.validate)(&val) {
                        None => Ok(val),
                        Some(err) => Err(err),
                    },
                }
            }
        }
    }

    pub mod map_err {
        use derivative::Derivative;

        use super::super::{Parser, ParserState};

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
                self.parser.parse(state).map_err(&self.transform)
            }
        }
    }

    pub mod backtracking {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct Backtracking<P> {
            parser: P,
        }

        impl<P> Backtracking<P> {
            pub fn new(parser: P) -> Self {
                Self { parser }
            }
        }

        impl<'a, P> Parser<'a> for Backtracking<P>
        where
            P: Parser<'a>,
        {
            type Item = P::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let backup = state.clone();
                match self.parser.parse(state) {
                    Ok(r) => Ok(r),
                    Err(err) => {
                        *state = backup;
                        Err(err)
                    }
                }
            }
        }
    }

    pub mod optional {
        use super::super::{Parser, ParserState};
        use std::marker::PhantomData;

        #[derive(Copy, Clone)]
        pub struct Optional<P, E> {
            parser: P,
            phantom: PhantomData<E>,
        }

        impl<P, E> Optional<P, E> {
            pub fn new(parser: P) -> Self {
                Self {
                    parser,
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, P, E> Parser<'a> for Optional<P, E>
        where
            P: Parser<'a>,
        {
            type Item = Option<P::Item>;

            type ParseError = E;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let backup = state.clone();
                Ok(match self.parser.parse(state) {
                    Ok(r) => Some(r),
                    Err(_) => {
                        *state = backup;
                        None
                    }
                })
            }
        }
    }

    pub mod looking_ahead {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct LookingAhead<P> {
            parser: P,
        }

        impl<P> LookingAhead<P> {
            pub fn new(parser: P) -> Self {
                Self { parser }
            }
        }

        impl<'a, P> Parser<'a> for LookingAhead<P>
        where
            P: Parser<'a>,
        {
            type Item = P::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                self.parser.parse(&mut state.clone())
            }
        }
    }

    pub mod falling_back {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct FallingBack<P, P2> {
            parser: P,
            fallback_parser: P2,
        }

        impl<P, P2> FallingBack<P, P2> {
            pub fn new(parser: P, fallback_parser: P2) -> Self {
                Self {
                    parser,
                    fallback_parser,
                }
            }
        }

        impl<'a, P, P2> Parser<'a> for FallingBack<P, P2>
        where
            P: Parser<'a>,
            P2: Parser<'a, Item = P::Item, ParseError = P::ParseError>,
        {
            type Item = P::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let previously_consumed = state.consumed_so_far;
                match self.parser.parse(state) {
                    Ok(result) => Ok(result),
                    Err(err) => {
                        if state.consumed_so_far == previously_consumed {
                            // main_parser did not consume any data, so we fallback
                            self.fallback_parser.parse(state)
                        } else {
                            Err(err)
                        }
                    }
                }
            }
        }
    }

    pub mod then {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct Then<P, P2> {
            parser: P,
            next_parser: P2,
        }

        impl<P, P2> Then<P, P2> {
            pub fn new(parser: P, next_parser: P2) -> Self {
                Self {
                    parser,
                    next_parser,
                }
            }
        }

        impl<'a, P, P2> Parser<'a> for Then<P, P2>
        where
            P: Parser<'a>,
            P2: Parser<'a, ParseError = P::ParseError>,
        {
            type Item = P2::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                self.parser.parse(state)?;
                self.next_parser.parse(state)
            }
        }
    }

    pub mod also {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct Also<P, P2> {
            parser: P,
            next_parser: P2,
        }

        impl<P, P2> Also<P, P2> {
            pub fn new(parser: P, next_parser: P2) -> Self {
                Self {
                    parser,
                    next_parser,
                }
            }
        }

        impl<'a, P, P2> Parser<'a> for Also<P, P2>
        where
            P: Parser<'a>,
            P2: Parser<'a, ParseError = P::ParseError>,
        {
            type Item = P::Item;

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let result = self.parser.parse(state)?;
                self.next_parser.parse(state)?;
                Ok(result)
            }
        }
    }

    pub mod paired_with {
        use super::super::{Parser, ParserState};

        #[derive(Copy, Clone)]
        pub struct PairedWith<P, P2> {
            first_parser: P,
            second_parser: P2,
        }

        impl<P, P2> PairedWith<P, P2> {
            pub fn new(first_parser: P, second_parser: P2) -> Self {
                Self {
                    first_parser,
                    second_parser,
                }
            }
        }

        impl<'a, P, P2> Parser<'a> for PairedWith<P, P2>
        where
            P: Parser<'a>,
            P2: Parser<'a, ParseError = P::ParseError>,
        {
            type Item = (P::Item, P2::Item);

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let first = self.first_parser.parse(state)?;
                let second = self.second_parser.parse(state)?;
                Ok((first, second))
            }
        }
    }

    pub mod bind {
        use derivative::Derivative;

        #[derive(Derivative)]
        #[derivative(Copy, Clone)]
        pub struct Bind<P, F> {
            parser: P,
            make_next_parser: F,
        }

        impl<P, F> Bind<P, F> {
            pub fn new(parser: P, make_next_parser: F) -> Self {
                Self {
                    parser,
                    make_next_parser,
                }
            }
        }
    }

    pub mod many {
        use std::marker::PhantomData;

        use derivative::Derivative;

        use super::{
            super::{Parser, ParserState},
            backtracking::Backtracking,
        };

        #[derive(Derivative)]
        #[derivative(Copy, Clone)]
        pub struct Many<P, E = !> {
            parser: Backtracking<P>,
            phantom: PhantomData<E>,
        }

        impl<'a, P, E> Many<P, E>
        where
            P: Parser<'a>,
        {
            pub fn new(parser: P) -> Self {
                Self {
                    parser: parser.backtracking(),
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, P, E> Parser<'a> for Many<P, E>
        where
            P: Parser<'a>,
        {
            type Item = Vec<P::Item>;

            type ParseError = E;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let mut results: Vec<P::Item> = vec![];
                while let Ok(item) = self.parser.parse(state) {
                    results.push(item);
                }
                Ok(results)
            }
        }
    }

    pub mod skip_many {
        use std::marker::PhantomData;

        use derivative::Derivative;

        use super::{
            super::{Parser, ParserState},
            backtracking::Backtracking,
        };

        #[derive(Derivative)]
        #[derivative(Copy, Clone)]
        pub struct SkipMany<P, E = !> {
            parser: Backtracking<P>,
            phantom: PhantomData<E>,
        }

        impl<'a, P, E> SkipMany<P, E>
        where
            P: Parser<'a>,
        {
            pub fn new(parser: P) -> Self {
                Self {
                    parser: parser.backtracking(),
                    phantom: PhantomData,
                }
            }
        }

        impl<'a, P, E> Parser<'a> for SkipMany<P, E>
        where
            P: Parser<'a>,
        {
            type Item = ();

            type ParseError = E;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                while let Ok(_) = self.parser.parse(state) {}
                Ok(())
            }
        }
    }

    pub mod at_least_one {
        use derivative::Derivative;

        use super::{
            super::{Parser, ParserState},
            many::Many,
        };

        #[derive(Derivative)]
        #[derivative(Copy, Clone)]
        pub struct AtLeastOne<P> {
            parse_one: P,
            parse_more: Many<P, !>,
        }

        impl<'a, P> AtLeastOne<P>
        where
            P: Parser<'a> + Clone,
        {
            pub fn new(parser: P) -> Self {
                Self {
                    parse_one: parser.clone(),
                    parse_more: parser.many(),
                }
            }
        }

        impl<'a, P> Parser<'a> for AtLeastOne<P>
        where
            P: Parser<'a>,
        {
            type Item = (P::Item, Vec<P::Item>);

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let first = self.parse_one.parse(state)?;
                // Safe to unwrap because parse_more returns Result<_, !>
                let more = self.parse_more.parse(state).unwrap();
                Ok((first, more))
            }
        }
    }

    pub mod with_span {
        use super::super::{Parser, ParserSpan, ParserState};

        #[derive(Copy, Clone)]
        pub struct WithSpan<P> {
            parser: P,
        }

        impl<P> WithSpan<P> {
            pub fn new(parser: P) -> Self {
                Self { parser }
            }
        }

        impl<'a, P> Parser<'a> for WithSpan<P>
        where
            P: Parser<'a>,
        {
            type Item = (P::Item, ParserSpan);

            type ParseError = P::ParseError;

            fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
                let start = state.current_position();
                self.parser.parse(state).map(|result| {
                    let end = state.current_position();
                    (result, (start, end))
                })
            }
        }
    }
}

// basic parsers

pub mod pure {
    use std::marker::PhantomData;

    use derivative::Derivative;

    use super::{Parser, ParserState};

    #[derive(Derivative)]
    #[derivative(Copy, Clone)]
    pub struct Borrowing<R, E = !> {
        val: R,
        _phantom: PhantomData<E>,
    }

    pub fn borrowing<R, E>(val: R) -> Borrowing<R, E> {
        Borrowing {
            val,
            _phantom: PhantomData,
        }
    }

    impl<'a, R, E> Parser<'a> for Borrowing<R, E>
    where
        R: 'a,
    {
        type Item = &'a R;
        type ParseError = E;
        fn parse(&'a self, _: &mut ParserState) -> Result<&'a R, E> {
            Ok(&self.val)
        }
    }

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

    impl<'a, R, E> Parser<'a> for Cloning<R, E>
    where
        R: Clone,
    {
        type Item = R;
        type ParseError = E;
        fn parse(&'a self, _: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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

    impl<'a, F, R, E> Parser<'a> for Lazy<F>
    where
        F: Fn() -> Result<R, E>,
    {
        type Item = R;
        type ParseError = E;
        fn parse(&'a self, _: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            (self.make_result)()
        }
    }
}

pub mod one_char {
    use std::fmt::Display;

    use super::{Parser, ParserState};

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

    impl<'a, Pred, Desc> Parser<'a> for OneCharMatches<Pred, Desc>
    where
        Pred: Fn(char) -> bool,
        Desc: Display,
    {
        type Item = char;
        type ParseError = String;
        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            match state.leftovers.chars().next() {
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

    pub static any: AnyChar = AnyChar {};

    impl<'a> Parser<'a> for AnyChar {
        type Item = char;
        type ParseError = String;
        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            match state.leftovers.chars().next() {
                Some(c) => {
                    state.advance(c.len_utf8());
                    Ok(c)
                }
                None => Err("Unexpected end of input.")?,
            }
        }
    }
}

pub mod string {
    use std::marker::PhantomData;

    use derivative::Derivative;

    use super::{Parser, ParserState};

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
    impl<'a, E> Parser<'a> for Check<'a, E> {
        type Item = bool;

        type ParseError = E;

        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            Ok(state.leftovers.starts_with(self.expected))
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
    impl<'a, E> Parser<'a> for CheckOwned<E> {
        type Item = bool;

        type ParseError = E;

        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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
    impl<'a> Parser<'a> for Expect<'a> {
        type Item = ();

        type ParseError = String;

        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            match check::<!>(self.expected).parse(state) {
                Ok(true) => {
                    state.advance(self.expected.len());
                    Ok(())
                }
                Ok(false) => Err(format!(
                    "Expected\n  {}\"but found\n  {}",
                    self.expected,
                    &state.leftovers[..self.expected.len()]
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
    impl<'a> Parser<'a> for ExpectOwned {
        type Item = ();

        type ParseError = String;

        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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

    impl<'a, Pred, E> Parser<'a> for ManyCharsMatching<Pred, E>
    where
        Pred: Fn(char) -> bool,
    {
        type Item = String;
        type ParseError = E;
        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            Ok({
                let mut s = String::new();
                s.extend(state.leftovers.chars().take_while(|c| (self.predicate)(*c)));
                s
            })
        }
    }
}

pub mod delim {
    use super::{Parser, ParserState};

    pub fn whitespace() -> super::one_char::OneCharMatches<fn(char) -> bool, &'static str> {
        super::one_char::matches(char::is_whitespace, "whitespace")
    }

    #[derive(Copy, Clone)]
    pub struct ExpectEnd {}

    pub static expect_end: ExpectEnd = ExpectEnd {};

    impl<'a> Parser<'a> for ExpectEnd {
        type Item = ();
        type ParseError = String;
        fn parse(&'a self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            if state.leftovers.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    "Expected end of input, but found {}",
                    &state.leftovers[..10]
                ))?
            }
        }
    }
}
