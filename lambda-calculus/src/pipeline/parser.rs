use crate::parsers::{delim, string, Parser};
use crate::pipeline::parse_tree::{Apply, Decl, Expr, Lambda, Stmt};
use once_cell::unsync::{Lazy, OnceCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct ParseStmt {
    expr_parser: Rc<OnceCell<Rc<dyn Parser<Item = Expr, ParseError = String>>>>,
}

impl ParseStmt {
    pub fn new() -> Self {
        Self {
            expr_parser: Rc::new(OnceCell::new()),
        }
    }

    pub fn parse(&self, text: &str) -> Result<Stmt, String> {
        self.parse_stmt().parse_str(text)
    }

    fn parse_stmt(&self) -> impl Parser<Item = Stmt, ParseError = String> + '_ {
        delim::whitespace()
            .skip_many::<String>()
            .then(
                (self.parse_decl())
                    .map(Stmt::Decl)
                    .falling_back((self.parse_expr()).map(Stmt::Expr)),
            )
            .also(delim::EXPECT_END)
    }

    fn parse_decl(&self) -> impl Parser<Item = Decl, ParseError = String> + '_ {
        string::expect("let")
            .then(delim::whitespace().skip_at_least_one())
            .then(self.parse_identifier())
            .also(
                delim::whitespace()
                    .skip_at_least_one()
                    .then(string::expect("="))
                    .then(delim::whitespace().skip_at_least_one()),
            )
            .paired_with(self.parse_expr())
            .also(delim::whitespace().skip_many())
            .map(|(identifier, expr)| Decl { identifier, expr })
    }

    fn parse_expr(&self) -> Rc<dyn Parser<Item = Expr, ParseError = String>> {
        let parse_lambda = {
            let self_ = self.clone();
            move || self_.parse_lambda()
        };
        let parse_application = {
            let self_ = self.clone();
            move || self_.parse_application()
        };
        self.expr_parser
            .get_or_init(|| {
                Rc::new(
                    Lazy::new(parse_lambda)
                        .falling_back(Lazy::new(parse_application))
                        .also(delim::whitespace().skip_many()),
                )
            })
            .clone()
    }

    fn parse_lambda(&self) -> impl Parser<Item = Expr, ParseError = String> {
        string::expect("\\")
            .also(delim::whitespace().skip_many())
            .then(self.parse_identifier())
            .also(delim::whitespace().skip_many())
            .also(string::expect("->"))
            .also(delim::whitespace().skip_many())
            .paired_with(self.parse_expr())
            .map(|(param, body)| Expr::Lambda(Box::new(Lambda { param, body })))
    }

    fn parse_application(&self) -> impl Parser<Item = Expr, ParseError = String> {
        Rc::new(
            self.parse_parens()
                .falling_back(self.parse_identifier().map(Expr::Lookup))
                .falling_back(self.parse_literal_integer().map(Expr::LitInteger)),
        )
        .at_least_one()
        // at the end of an application series, we may have a trailing lambda
        // e.g. f +1 +2 +3 \k -> +4
        .paired_with(self.parse_lambda().optional())
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

    fn parse_parens(&self) -> impl Parser<Item = Expr, ParseError = String> {
        string::expect("(")
            .also(delim::whitespace().skip_many())
            .then(self.parse_expr())
            .also(delim::whitespace().skip_many())
            .also(string::expect(")"))
            .also(delim::whitespace().skip_many())
    }

    fn parse_identifier(&self) -> impl Parser<Item = String, ParseError = String> {
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

    fn parse_literal_integer(&self) -> impl Parser<Item = i64, ParseError = String> {
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
}
