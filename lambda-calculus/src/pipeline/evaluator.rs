use std::rc::Rc;

use super::{
    locally_nameless_tree::{Apply, BoundVar, Expr, FreeVar, Lambda, Lookup},
    runtime::{Globals, Locals, RTValue},
};

struct BuildError {
    msg: String,
}

pub trait CompiledClosure {
    fn run(&self, globals: &Globals, locals: Locals) -> Result<RTValue, String>;
}

impl<T> CompiledClosure for T
where
    T: Clone + Fn(&Globals, Locals) -> Result<RTValue, String>,
{
    fn run(&self, globals: &Globals, locals: Locals) -> Result<RTValue, String> {
        self(globals, locals)
    }
}

pub fn eval(expr: &Expr, globals: &Globals) -> Result<RTValue, String> {
    match go_expr(expr) {
        Ok(expr) => expr.run(globals, Locals::empty()),
        Err(build_error) => Err(format!("Build error: {}", build_error.msg)),
    }
}

fn go_expr(expr: &Expr) -> Result<Box<dyn CompiledClosure>, BuildError> {
    Ok(match expr {
        Expr::Lambda(lambda) => Box::new(go_lambda(lambda)?),
        Expr::Apply(apply) => Box::new(go_apply(apply)?),
        Expr::Lookup(lookup) => go_lookup(lookup)?,
        Expr::LitInteger(i) => {
            let i = *i;
            Box::new(move |_: &Globals, _: Locals| Ok(RTValue::Integer(i)))
        }
    })
}

fn go_lambda(lambda: &Lambda) -> Result<impl Clone + CompiledClosure, BuildError> {
    let run_body: Rc<dyn CompiledClosure> = Rc::from(go_expr(&lambda.body)?);
    Ok(move |_: &Globals, locals: Locals| {
        let run_body = run_body.clone();
        Ok(RTValue::Function(Rc::new(move |arg, globals| {
            run_body.run(globals, locals.bind_local(arg))
        })))
    })
}

fn go_apply(
    Apply { function, argument }: &Apply,
) -> Result<impl Clone + CompiledClosure, BuildError> {
    let function: Rc<dyn CompiledClosure> = Rc::from(go_expr(function)?);
    let argument: Rc<dyn CompiledClosure> = Rc::from(go_expr(argument)?);
    Ok(
        move |globals: &Globals, locals: Locals| match function.run(globals, locals.clone())? {
            RTValue::Function(run_fn) => {
                let arg = argument.run(globals, locals)?;
                (*run_fn)(arg, globals)
            }
            RTValue::Integer(i) => Err(format!("Cannot call number {} as a fn", i)),
        },
    )
}

fn go_lookup(lookup: &Lookup) -> Result<Box<dyn CompiledClosure>, BuildError> {
    match lookup {
        Lookup::Free(FreeVar { name }) => {
            let name = name.clone();
            Ok(Box::new(move |globals: &Globals, _: Locals| {
                globals
                    .lookup(&name[..])
                    .ok_or_else(|| format!("Unbound global {}", name))
            }))
        }

        Lookup::Bound(BoundVar { de_brujn_index }) => {
            let de_brujn_index = *de_brujn_index;
            Ok(Box::new(move |_: &Globals, locals: Locals| {
                locals
                    .lookup(de_brujn_index)
                    .ok_or_else(|| format!("Unbound local index {}", de_brujn_index))
            }))
        }
    }
}
