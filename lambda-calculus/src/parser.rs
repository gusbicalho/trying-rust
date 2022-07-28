use std::rc::Rc;
mod internals;
use internals as parser;
use once_cell::unsync::{Lazy, OnceCell};
pub use parser::Parser;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Decl(Decl),
}

#[derive(Debug)]
pub struct Decl {
    pub identifier: String,
    pub expr: Expr,
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
    pub param: String,
    pub body: Expr,
}

#[derive(Debug)]
pub struct Apply {
    pub function: Expr,
    pub argument: Expr,
}

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
        parser::delim::whitespace()
            .skip_many::<String>()
            .then(
                (self.parse_decl())
                    .map(Stmt::Decl)
                    .falling_back((self.parse_expr()).map(Stmt::Expr)),
            )
            .also(parser::delim::EXPECT_END)
    }

    fn parse_decl(&self) -> impl Parser<Item = Decl, ParseError = String> + '_ {
        parser::string::expect("let")
            .also(parser::pure::lazy(|| {
                println!("parsed let");
                Ok(())
            }))
            .then(parser::delim::whitespace().skip_at_least_one())
            .then(self.parse_identifier())
            .also(
                parser::delim::whitespace()
                    .skip_at_least_one()
                    .then(parser::string::expect("="))
                    .then(parser::delim::whitespace().skip_at_least_one()),
            )
            .paired_with(self.parse_expr())
            .also(parser::delim::whitespace().skip_many())
            .map(|(identifier, expr)| Decl { identifier, expr })
    }

    fn parse_expr(&self) -> Rc<dyn Parser<Item = Expr, ParseError = String>> {
        let parse_lambda = {
            let self2 = self.clone();
            move || self2.parse_lambda()
        };
        let self3 = self.clone();
        let parse_application = move || self3.parse_application();
        self.expr_parser
            .get_or_init(|| {
                Rc::new(
                    Lazy::new(parse_lambda)
                        .falling_back(Lazy::new(parse_application))
                        .also(parser::delim::whitespace().skip_many()),
                )
            })
            .clone()
    }

    fn parse_lambda(&self) -> impl Parser<Item = Expr, ParseError = String> {
        parser::string::expect("\\")
            .also(parser::pure::lazy(|| {
                println!("parsed \\");
                Ok(())
            }))
            .also(parser::delim::whitespace().skip_many())
            .then(self.parse_identifier())
            .also(parser::delim::whitespace().skip_many())
            .also(parser::string::expect("->"))
            .also(parser::delim::whitespace().skip_many())
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
        parser::string::expect("(")
            .also(parser::pure::lazy(|| {
                println!("parsed (");
                Ok(())
            }))
            .also(parser::delim::whitespace().skip_many())
            .then(self.parse_expr())
            .also(parser::delim::whitespace().skip_many())
            .also(parser::string::expect(")"))
            .also(parser::delim::whitespace().skip_many())
    }

    fn parse_identifier(&self) -> impl Parser<Item = String, ParseError = String> {
        parser::string::many_chars_matching(|c| char::is_ascii_lowercase(&c))
            .validate(|identifier| {
                println!("parsed identifier: {}", identifier);
                if identifier.is_empty() {
                    Some("Expected identifier (sequence of lowercase ascii letters)".to_string())
                } else {
                    None
                }
            })
            .also(parser::delim::whitespace().skip_many())
    }

    fn parse_literal_integer(&self) -> impl Parser<Item = i64, ParseError = String> {
        parser::string::expect("-")
            .map(|_| false)
            .falling_back(parser::string::expect("+").map(|_| true))
            .also(parser::pure::lazy(|| {
                println!("parsed number sign");
                Ok(())
            }))
            .paired_with(
                parser::string::many_chars_matching(|c: char| c.is_ascii_digit()).validate(
                    |digits| {
                        if digits.is_empty() {
                            Some("Expected sequence of digits".to_string())
                        } else {
                            None
                        }
                    },
                ),
            )
            .also(parser::delim::whitespace().skip_many())
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

// fn parse_stmt() -> impl Parser<Item = Stmt, ParseError = String> {
//     parser::delim::whitespace().skip_many::<String>().then(
//         (parse_decl())
//             .map(Stmt::Decl)
//             .falling_back((parse_expr()).map(Stmt::Expr)),
//     )
// }

// fn parse_decl() -> impl Parser<Item = Decl, ParseError = String> {
//     parser::string::expect("let")
//         .also(parser::pure::lazy(|| {
//             println!("parsed let");
//             Ok(())
//         }))
//         .then(parser::delim::whitespace().skip_at_least_one())
//         .then(parse_identifier())
//         .also(
//             parser::delim::whitespace()
//                 .skip_at_least_one()
//                 .then(parser::string::expect("="))
//                 .then(parser::delim::whitespace().skip_at_least_one()),
//         )
//         .paired_with(parse_expr())
//         .also(
//             parser::delim::whitespace()
//                 .skip_many()
//                 .then(parser::delim::EXPECT_END),
//         )
//         .map(|(identifier, expr)| Decl { identifier, expr })
// }

// pub fn parse_expr() -> Box<dyn Parser<Item = Expr, ParseError = String>> {
//     Box::new(parse_lambda().falling_back(parse_application()))
// }

// pub fn parse_lambda() -> impl Parser<Item = Expr, ParseError = String> {
//     parser::string::expect("\\")
//         .also(parser::pure::lazy(|| {
//             println!("parsed \\");
//             Ok(())
//         }))
//         .also(parser::delim::whitespace().skip_many())
//         .then(parse_identifier())
//         .also(parser::delim::whitespace().skip_many())
//         .also(parser::string::expect("->"))
//         .also(parser::delim::whitespace().skip_many())
//         .paired_with(parse_expr())
//         .map(|(param, body)| Expr::Lambda(Box::new(Lambda { param, body })))
// }

// pub fn parse_application() -> impl Parser<Item = Expr, ParseError = String> {
//     Rc::new(
//         parse_parens()
//             .falling_back(parse_identifier().map(Expr::Lookup))
//             .falling_back(parse_literal_integer().map(Expr::LitInteger)),
//     )
//     .at_least_one()
//     .map(|(head, args)| {
//         args.into_iter().fold(head, |head, arg| {
//             Expr::Apply(Box::new(Apply {
//                 function: head,
//                 argument: arg,
//             }))
//         })
//     })
// }

// pub fn parse_parens() -> impl Parser<Item = Expr, ParseError = String> {
//     parser::string::expect("(")
//         .also(parser::pure::lazy(|| {
//             println!("parsed (");
//             Ok(())
//         }))
//         .also(parser::delim::whitespace().skip_many())
//         .then(parse_expr())
//         .also(parser::delim::whitespace().skip_many())
//         .also(parser::string::expect(")"))
// }

// pub fn parse_identifier() -> impl Parser<Item = String, ParseError = String> {
//     parser::string::many_chars_matching(|c| char::is_ascii_lowercase(&c)).validate(|identifier| {
//         println!("parsed identifier: {}", identifier);
//         if identifier.is_empty() {
//             Some("Expected identifier (sequence of lowercase ascii letters)".to_string())
//         } else {
//             None
//         }
//     })
// }

// pub fn parse_literal_integer() -> impl Parser<Item = i64, ParseError = String> {
//     parser::string::expect("-")
//         .map(|_| false)
//         .falling_back(parser::string::expect("+").map(|_| true))
//         .also(parser::pure::lazy(|| {
//             println!("parsed number sign");
//             Ok(())
//         }))
//         .paired_with(
//             parser::string::many_chars_matching(|c: char| c.is_ascii_digit()).validate(|digits| {
//                 if digits.is_empty() {
//                     Some("Expected sequence of digits".to_string())
//                 } else {
//                     None
//                 }
//             }),
//         )
//         .map(|(is_positive, digits)| {
//             let abs_val = digits.parse::<i64>().unwrap_or(0);
//             if is_positive {
//                 abs_val
//             } else {
//                 -abs_val
//             }
//         })
// }
