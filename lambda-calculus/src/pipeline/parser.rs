use crate::parsers::{delim, pure, string, Parser, ParserState};
use crate::pipeline::parse_tree::{Apply, Decl, Expr, Lambda, Stmt};
use std::rc::Rc;

pub fn parse_stmt(text: &str) -> Result<Stmt, String> {
    stmt().parse_str(text)
}

fn stmt() -> impl Parser<Item = Stmt, ParseError = String> {
    delim::whitespace()
        .skip_many::<String>()
        .then(
            (decl())
                .map(Stmt::Decl)
                .falling_back(expr().map(Stmt::Expr)),
        )
        .also(delim::EXPECT_END)
}

fn decl() -> impl Parser<Item = Decl, ParseError = String> {
    string::expect("let")
        .then(delim::whitespace().skip_at_least_one())
        .then(identifier())
        .also(
            delim::whitespace()
                .skip_at_least_one()
                .then(string::expect("="))
                .then(delim::whitespace().skip_at_least_one()),
        )
        .paired_with(expr())
        .also(delim::whitespace().skip_many())
        .map(|(identifier, expr)| Decl { identifier, expr })
}

fn expr() -> impl Parser<Item = Expr, ParseError = String> {
    lambda()
        .falling_back(application())
        .also(delim::whitespace().skip_many())
}

fn lambda() -> impl Parser<Item = Expr, ParseError = String> {
    string::expect("\\")
        .also(delim::whitespace().skip_many())
        .then(identifier())
        .also(delim::whitespace().skip_many())
        .also(string::expect("->"))
        .also(delim::whitespace().skip_many())
        .paired_with(pure::run(|state| expr().parse(state)))
        .map(|(param, body)| Expr::Lambda(Box::new(Lambda { param, body })))
}

fn application() -> impl Parser<Item = Expr, ParseError = String> {
    Rc::new(
        parens(pure::run(|state| expr().parse(state)))
            .falling_back(identifier().map(Expr::Lookup))
            .falling_back(literal_integer().map(Expr::LitInteger)),
    )
    .at_least_one()
    // at the end of an application series, we may have a trailing lambda
    // e.g. f +1 +2 +3 \k -> +4
    .paired_with(lambda().optional())
    .map(|((head, mut args), final_lambda)| {
        if let Some(lambda_expr) = final_lambda {
            args.push(lambda_expr);
        }
        args.into_iter().fold(head, |head, arg| {
            Expr::Apply(Box::new(Apply {
                function: head,
                argument: arg,
            }))
        })
    })
}

fn parens<P>(parse_item: P) -> impl Parser<Item = P::Item, ParseError = String>
where
    P: Parser<ParseError = String>,
{
    string::expect("(")
        .also(delim::whitespace().skip_many())
        .then(parse_item)
        .also(delim::whitespace().skip_many())
        .also(string::expect(")"))
        .also(delim::whitespace().skip_many())
}

fn identifier() -> impl Parser<Item = String, ParseError = String> {
    string::many_chars_matching(|c| char::is_ascii_lowercase(&c))
        .validate(|identifier| {
            if identifier.is_empty() {
                Some("Expected identifier (sequence of lowercase ascii letters)".to_string())
            } else {
                None
            }
        })
        .also(delim::whitespace().skip_many())
}

fn literal_integer() -> impl Parser<Item = i64, ParseError = String> {
    string::expect("-")
        .map(|_| false)
        .falling_back(string::expect("+").map(|_| true))
        .paired_with(
            string::many_chars_matching(|c: char| c.is_ascii_digit()).validate(|digits| {
                if digits.is_empty() {
                    Some("Expected sequence of digits".to_string())
                } else {
                    None
                }
            }),
        )
        .also(delim::whitespace().skip_many())
        .map(|(is_positive, digits)| {
            let abs_val = digits.parse::<i64>().unwrap_or(0);
            if is_positive {
                abs_val
            } else {
                -abs_val
            }
        })
}
