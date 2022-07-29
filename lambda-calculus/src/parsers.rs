mod internals;
pub use internals::{Parser, ParserState, ParserSpan, ParserPos};

pub mod adapters;
pub mod delim;
pub mod one_char;
pub mod pure;
pub mod string;
