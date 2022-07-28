use std::rc::Rc;
mod internals;
use internals as parser;
pub use parser::Parser;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Decl(Decl),
}

#[derive(Debug)]
pub struct Decl {
    identifier: String,
    expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Lambda(Box<Lambda>),
    Apply(Box<Apply>),
    Lookup(String),
    LitInteger(i64),
}

#[derive(Debug)]
pub struct Lambda {
    param: String,
    body: Expr,
}

#[derive(Debug)]
pub struct Apply {
    function: Expr,
    argument: Expr,
}

pub fn parse_stmt<'a>() -> impl Parser<'a, Item = Stmt, ParseError = String> {
    parser::delim::whitespace().skip_many::<String>().then(
        (parse_decl())
            .map(Stmt::Decl)
            .falling_back((parse_expr()).map(Stmt::Expr)),
    )
}

pub fn parse_decl<'a>() -> impl Parser<'a, Item = Decl, ParseError = String> {
    parser::string::expect("let")
        .then(parser::delim::whitespace().skip_at_least_one())
        .then(parse_identifier())
        .also(
            parser::delim::whitespace()
                .skip_at_least_one()
                .then(parser::string::expect("="))
                .then(parser::delim::whitespace().skip_at_least_one()),
        )
        .paired_with(parse_expr())
        .also(
            parser::delim::whitespace()
                .skip_many()
                .then(parser::delim::expect_end),
        )
        .map(|(identifier, expr)| Decl { identifier, expr })
}

pub fn parse_expr<'a>() -> Box<dyn Parser<'a, Item = Expr, ParseError = String>> {
    Box::new(parse_lambda().falling_back(parse_application()))
}

pub fn parse_lambda<'a>() -> impl Parser<'a, Item = Expr, ParseError = String> {
    parser::string::expect("\\")
        .also(parser::delim::whitespace().skip_many())
        .then(parse_identifier())
        .also(parser::delim::whitespace().skip_many())
        .also(parser::string::expect("->"))
        .also(parser::delim::whitespace().skip_many())
        .paired_with(parse_expr())
        .map(|(param, body)| Expr::Lambda(Box::new(Lambda { param, body })))
}

pub fn parse_application<'a>() -> impl Parser<'a, Item = Expr, ParseError = String> {
    Rc::new(
        parse_parens()
            .falling_back(parse_identifier().map(Expr::Lookup))
            .falling_back(parse_literal_integer().map(Expr::LitInteger)),
    )
    .at_least_one()
    .map(|(head, args)| {
        args.into_iter().fold(head, |head, arg| {
            Expr::Apply(Box::new(Apply {
                function: head,
                argument: arg,
            }))
        })
    })
}

pub fn parse_parens<'a>() -> impl Parser<'a, Item = Expr, ParseError = String> {
    parser::string::expect("(")
        .also(parser::delim::whitespace().skip_many())
        .then(parse_expr())
        .also(parser::delim::whitespace().skip_many())
        .also(parser::string::expect(")"))
}

pub fn parse_identifier<'a>() -> impl Parser<'a, Item = String, ParseError = String> {
    parser::string::many_chars_matching(|c| char::is_ascii_lowercase(&c)).validate(|identifier| {
        if identifier.is_empty() {
            Some("Expected identifier (sequence of lowercase ascii letters)".to_string())
        } else {
            None
        }
    })
}

pub fn parse_literal_integer<'a>() -> impl Parser<'a, Item = i64, ParseError = String> {
    parser::string::expect("_").optional().map(|_| 42i64)
}
