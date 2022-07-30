#[derive(Debug, Clone)]
pub enum Expr {
    Lambda(Box<Lambda>),
    Apply(Box<Apply>),
    Lookup(Lookup),
    LitInteger(i64),
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub body: Expr,
}

impl Lambda {
    pub fn new(body: Expr) -> Self {
        Self { body }
    }
}

#[derive(Debug, Clone)]
pub struct Apply {
    pub function: Expr,
    pub argument: Expr,
}

impl Apply {
    pub fn new(function: Expr, argument: Expr) -> Self {
        Self { function, argument }
    }
}

#[derive(Debug, Clone)]
pub enum Lookup {
    Free(FreeVar),
    Bound(BoundVar),
}

#[derive(Debug, Clone)]
pub struct FreeVar {
    pub name: String,
}

impl FreeVar {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub struct BoundVar {
    pub de_brujn_index: usize,
}

impl BoundVar {
    pub fn new(de_brujn_index: usize) -> Self {
        Self { de_brujn_index }
    }
}
