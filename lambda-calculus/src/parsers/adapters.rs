pub mod map {
    use derivative::Derivative;

    use super::super::internals::{Parser, ParserState};

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

    impl<P, F, I2> Parser for Map<P, F>
    where
        P: Parser,
        F: Fn(P::Item) -> I2,
    {
        type Item = I2;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            self.parser.parse(state).map(&self.transform)
        }
    }
}

pub mod validate {
    use derivative::Derivative;

    use super::super::internals::{Parser, ParserState};

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

    impl<P, F> Parser for Validate<P, F>
    where
        P: Parser,
        F: Fn(&P::Item) -> Option<P::ParseError>,
    {
        type Item = P::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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

    use super::super::internals::{Parser, ParserState};

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

    impl<P, F, E2> Parser for MapErr<P, F>
    where
        P: Parser,
        F: Fn(P::ParseError) -> E2,
    {
        type Item = P::Item;

        type ParseError = E2;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            self.parser.parse(state).map_err(&self.transform)
        }
    }
}

pub mod backtracking {
    use super::super::internals::{Parser, ParserState};

    #[derive(Copy, Clone)]
    pub struct Backtracking<P> {
        parser: P,
    }

    impl<P> Backtracking<P> {
        pub fn new(parser: P) -> Self {
            Self { parser }
        }
    }

    impl<P> Parser for Backtracking<P>
    where
        P: Parser,
    {
        type Item = P::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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
    use super::super::internals::{Parser, ParserState};

    #[derive(Copy, Clone)]
    pub struct Optional<P> {
        parser: P,
    }

    impl<P> Optional<P> {
        pub fn new(parser: P) -> Self {
            Self { parser }
        }
    }

    impl<P> Parser for Optional<P>
    where
        P: Parser,
    {
        type Item = Option<P::Item>;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let previously_consumed = state.consumed_so_far();
            match self.parser.parse(state) {
                Ok(r) => Ok(Some(r)),
                Err(err) => {
                    if state.consumed_so_far() == previously_consumed {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                }
            }
        }
    }
}

pub mod looking_ahead {
    use super::super::internals::{Parser, ParserState};

    #[derive(Copy, Clone)]
    pub struct LookingAhead<P> {
        parser: P,
    }

    impl<P> LookingAhead<P> {
        pub fn new(parser: P) -> Self {
            Self { parser }
        }
    }

    impl<P> Parser for LookingAhead<P>
    where
        P: Parser,
    {
        type Item = P::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            self.parser.parse(&mut state.clone())
        }
    }
}

pub mod falling_back {
    use super::super::internals::{Parser, ParserState};

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

    impl<P, P2> Parser for FallingBack<P, P2>
    where
        P: Parser,
        P2: Parser<Item = P::Item, ParseError = P::ParseError>,
    {
        type Item = P::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let previously_consumed = state.consumed_so_far();
            match self.parser.parse(state) {
                Ok(result) => Ok(result),
                Err(err) => {
                    if state.consumed_so_far() == previously_consumed {
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
    use super::super::internals::{Parser, ParserState};

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

    impl<P, P2> Parser for Then<P, P2>
    where
        P: Parser,
        P2: Parser<ParseError = P::ParseError>,
    {
        type Item = P2::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            self.parser.parse(state)?;
            self.next_parser.parse(state)
        }
    }
}

pub mod also {
    use super::super::internals::{Parser, ParserState};

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

    impl<P, P2> Parser for Also<P, P2>
    where
        P: Parser,
        P2: Parser<ParseError = P::ParseError>,
    {
        type Item = P::Item;

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let result = self.parser.parse(state)?;
            self.next_parser.parse(state)?;
            Ok(result)
        }
    }
}

pub mod paired_with {
    use super::super::internals::{Parser, ParserState};

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

    impl<P, P2> Parser for PairedWith<P, P2>
    where
        P: Parser,
        P2: Parser<ParseError = P::ParseError>,
    {
        type Item = (P::Item, P2::Item);

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let first = self.first_parser.parse(state)?;
            let second = self.second_parser.parse(state)?;
            Ok((first, second))
        }
    }
}

pub mod many {
    use std::marker::PhantomData;

    use derivative::Derivative;

    use super::{
        super::internals::{Parser, ParserState},
        backtracking::Backtracking,
    };

    #[derive(Derivative)]
    #[derivative(Copy, Clone)]
    pub struct Many<P, E = !> {
        parser: Backtracking<P>,
        phantom: PhantomData<E>,
    }

    impl<P, E> Many<P, E>
    where
        P: Parser,
    {
        pub fn new(parser: P) -> Self {
            Self {
                parser: parser.backtracking(),
                phantom: PhantomData,
            }
        }
    }

    impl<P, E> Parser for Many<P, E>
    where
        P: Parser,
    {
        type Item = Vec<P::Item>;

        type ParseError = E;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
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
        super::internals::{Parser, ParserState},
        backtracking::Backtracking,
    };

    #[derive(Derivative)]
    #[derivative(Copy, Clone)]
    pub struct SkipMany<P, E = !> {
        parser: Backtracking<P>,
        phantom: PhantomData<E>,
    }

    impl<P, E> SkipMany<P, E>
    where
        P: Parser,
    {
        pub fn new(parser: P) -> Self {
            Self {
                parser: parser.backtracking(),
                phantom: PhantomData,
            }
        }
    }

    impl<P, E> Parser for SkipMany<P, E>
    where
        P: Parser,
    {
        type Item = ();

        type ParseError = E;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            while self.parser.parse(state).is_ok() {}
            Ok(())
        }
    }
}

pub mod at_least_one {
    use derivative::Derivative;

    use super::{
        super::internals::{Parser, ParserState},
        many::Many,
    };

    #[derive(Derivative)]
    #[derivative(Copy, Clone)]
    pub struct AtLeastOne<P> {
        parse_one: P,
        parse_more: Many<P, !>,
    }

    impl<P> AtLeastOne<P>
    where
        P: Parser + Clone,
    {
        pub fn new(parser: P) -> Self {
            Self {
                parse_one: parser.clone(),
                parse_more: parser.many(),
            }
        }
    }

    impl<P> Parser for AtLeastOne<P>
    where
        P: Parser,
    {
        type Item = (P::Item, Vec<P::Item>);

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let first = self.parse_one.parse(state)?;
            // Safe to unwrap because parse_more returns Result<_, !>
            let more = self.parse_more.parse(state).unwrap();
            Ok((first, more))
        }
    }
}

pub mod with_span {
    use super::super::internals::{Parser, ParserSpan, ParserState};

    #[derive(Copy, Clone)]
    pub struct WithSpan<P> {
        parser: P,
    }

    impl<P> WithSpan<P> {
        pub fn new(parser: P) -> Self {
            Self { parser }
        }
    }

    impl<P> Parser for WithSpan<P>
    where
        P: Parser,
    {
        type Item = (P::Item, ParserSpan);

        type ParseError = P::ParseError;

        fn parse(&self, state: &mut ParserState) -> Result<Self::Item, Self::ParseError> {
            let start = state.current_position().to_owned();
            self.parser
                .parse(state)
                .map(|result| (result, (start, state.current_position().to_owned())))
        }
    }
}
