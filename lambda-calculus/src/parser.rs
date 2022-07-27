use std::error::Error;
mod internals;
use internals::ParserState;
mod internals2;

use self::internals::{IsParserFn, Parser};

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

pub fn parse_stmt(text: String) -> Result<Stmt, Box<dyn Error>> {
    ParserState::new(&text).run(
        &parse_decl()
            .map(Stmt::Decl)
            .falling_back(&parse_expr().map(Stmt::Expr)),
    )
}

pub fn parse_decl() -> Parser<impl IsParserFn<Decl>, Decl> {
    // let whitespace = internals::expect(" ").skip_many();
    // let whitespace1 = internals::expect(" ").skip_at_least_one();
    // let parser = internals::expect("let")
    //     .followed_by(&whitespace1)
    //     .followed_by(&parse_identifier())
    //     .and_then(|identifier| {
    //         whitespace1
    //             .followed_by(&internals::expect("="))
    //             .followed_by(&whitespace1)
    //             .followed_by(&parse_expr())
    //             .and_then(|expr| {
    //                 whitespace
    //                     .followed_by(&internals::expect_end())
    //                     .map(|_| Decl { identifier, expr })
    //             })
    //     });
    // Parser::new(move |state: &mut ParserState| {
    //     state.run(&parser)
    // })
    todo!()
}

pub fn parse_expr() -> Parser<impl IsParserFn<Expr>, Expr> {
    todo!()
}

// pub fn parse_expr(state: &mut ParserState) -> Result<Expr, Box<dyn Error>> {
//     // state.run(internals::falling_back(
//     //     internals::map(parse_lambda, |l| Expr::Lambda(Box::new(l))),
//     //     internals::falling_back(
//     //         internals::map(parse_apply, |a| Expr::Apply(Box::new(a))),
//     //         internals::falling_back(
//     //             internals::map(parse_identifier, Expr::Lookup),
//     //             internals::map(parse_literal_integer, Expr::LitInteger),
//     //         ),
//     //     ),
//     // ))
//     todo!()
// }

pub fn parse_lambda(state: &mut ParserState) -> Result<Lambda, Box<dyn Error>> {
    todo!()
}
pub fn parse_apply(state: &mut ParserState) -> Result<Apply, Box<dyn Error>> {
    todo!()
}

pub fn parse_identifier() -> Parser<impl IsParserFn<String>, String> {
    todo!()
}
// pub fn parse_identifier(state: &mut ParserState) -> Result<String, Box<dyn Error>> {
//     // state.run(internals::map(
//     //     internals::at_least_one(internals::one_char(
//     //         |c| char::is_ascii_lowercase(&c),
//     //         "a lowercase ascii letter (a-z)",
//     //     )),
//     //     |chars| chars.iter().collect::<String>(),
//     // ))
//     todo!()
// }
pub fn parse_literal_integer(state: &mut ParserState) -> Result<i64, Box<dyn Error>> {
    todo!()
}
