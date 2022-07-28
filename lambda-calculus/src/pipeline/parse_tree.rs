#![allow(dead_code)]

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
