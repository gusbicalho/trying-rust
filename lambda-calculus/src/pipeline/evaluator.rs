use std::rc::Rc;

use super::{
    locally_nameless_tree::{Apply, BoundVar, Expr, FreeVar, Lambda, Lookup},
    runtime::{Globals, Locals, RTFunction, RTValue},
};

pub fn eval(expr: &Expr, globals: &Globals) -> Result<RTValue, String> {
    go_expr(expr, globals, Locals::empty())
}

fn go_expr(expr: &Expr, globals: &Globals, locals: Locals) -> Result<RTValue, String> {
    Ok(match expr {
        Expr::Lambda(lambda) => RTValue::Function(go_lambda(lambda, locals)?),
        Expr::Apply(apply) => go_apply(apply, globals, locals)?,
        Expr::Lookup(lookup) => go_lookup(lookup, globals, locals)?,
        Expr::LitInteger(i) => RTValue::Integer(*i),
    })
}

fn go_lambda(lambda: &Lambda, locals: Locals) -> Result<RTFunction, String> {
    let body = lambda.body.clone();
    Ok(Rc::new(move |arg, globals| {
        go_expr(&body, globals, locals.bind_local(arg))
    }))
}

fn go_apply(
    Apply { function, argument }: &Apply,
    globals: &Globals,
    locals: Locals,
) -> Result<RTValue, String> {
    match go_expr(function, globals, locals.clone())? {
        RTValue::Function(run_fn) => {
            let arg = go_expr(argument, globals, locals)?;
            (*run_fn)(arg, globals)
        }
        RTValue::Integer(i) => Err(format!("Cannot call number {} as a fn", i)),
    }
}

fn go_lookup(lookup: &Lookup, globals: &Globals, locals: Locals) -> Result<RTValue, String> {
    match lookup {
        Lookup::Free(FreeVar { name }) => globals
            .lookup(name)
            .ok_or_else(|| format!("Unbound global {}", name)),
        Lookup::Bound(BoundVar { de_brujn_index }) => locals
            .lookup(*de_brujn_index)
            .ok_or_else(|| format!("Unbound local index {}", de_brujn_index)),
    }
}
