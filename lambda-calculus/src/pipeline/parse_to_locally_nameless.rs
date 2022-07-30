use super::parse_tree;
use super::locally_nameless_tree;

pub fn transform_expr(parsed: &parse_tree::Expr) -> locally_nameless_tree::Expr {
    let mut bound_vars: Vec<&str> = vec![];
    go_expr(parsed, &mut bound_vars)
}

fn go_expr<'a>(parsed: &'a parse_tree::Expr, bound_vars: &mut Vec<&'a str>) -> locally_nameless_tree::Expr {
    match parsed {
        parse_tree::Expr::Apply(apply) => go_apply(apply, bound_vars),
        parse_tree::Expr::Lambda(lambda) => go_lambda(lambda, bound_vars),
        parse_tree::Expr::Lookup(identifier) => go_lookup(identifier, bound_vars),
        parse_tree::Expr::LitInteger(lit) => go_literal(lit),
    }
}

fn go_apply<'a>(
    parsed: &'a parse_tree::Apply,
    bound_vars: &mut Vec<&'a str>,
) -> locally_nameless_tree::Expr {
    let function = go_expr(&parsed.function, bound_vars);
    let argument = go_expr(&parsed.argument, bound_vars);
    locally_nameless_tree::Expr::Apply(Box::new(locally_nameless_tree::Apply::new(function, argument)))
}

fn go_lambda<'a>(
    parsed: &'a parse_tree::Lambda,
    bound_vars: &mut Vec<&'a str>,
) -> locally_nameless_tree::Expr {
    bound_vars.push(&parsed.param[..]);
    let body = go_expr(&parsed.body, bound_vars);
    bound_vars.pop();
    locally_nameless_tree::Expr::Lambda(Box::new(locally_nameless_tree::Lambda::new(body)))
}

fn go_lookup(identifier: &String, bound_vars: &[&str]) -> locally_nameless_tree::Expr {
    locally_nameless_tree::Expr::Lookup(
        match bound_vars.iter().rev().position(|s| s == identifier) {
            None => locally_nameless_tree::Lookup::Free(locally_nameless_tree::FreeVar::new(identifier.clone())),
            Some(i) => locally_nameless_tree::Lookup::Bound(locally_nameless_tree::BoundVar::new(i)),
        },
    )
}

fn go_literal(literal: &i64) -> locally_nameless_tree::Expr {
    locally_nameless_tree::Expr::LitInteger(*literal)
}
