use std::{collections::HashMap, fmt::Display, rc::Rc};

pub struct Globals {
    globals: HashMap<String, RTValue>,
}

impl Globals {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    pub fn lookup(&self, name: &str) -> Option<RTValue> {
        self.globals.get(name).map(Clone::clone)
    }

    pub fn define(&mut self, name: &str, val: RTValue) {
        self.globals.insert(name.to_string(), val);
    }
}

#[derive(Clone)]
pub struct Locals {
    locals: Option<Rc<RTFrame>>,
}

impl Locals {
    pub fn empty() -> Self {
        Self { locals: None }
    }
    pub fn bind_local(&self, arg: RTValue) -> Locals {
        Self {
            locals: Some(Rc::new(RTFrame {
                var: arg,
                parent: self.locals.clone(),
            })),
        }
    }

    pub fn lookup(&self, mut de_brujn_index: usize) -> Option<RTValue> {
        self.locals.clone().and_then(|mut frame| {
            while de_brujn_index != 0 {
                match &frame.parent {
                    None => return None,
                    Some(parent) => {
                        frame = parent.clone();
                        de_brujn_index -= 1;
                    }
                }
            }
            Some(frame.var.clone())
        })
    }
}

#[derive(Clone)]
pub struct RTFrame {
    pub var: RTValue,
    pub parent: Option<Rc<RTFrame>>,
}

#[derive(Clone)]
pub enum RTValue {
    Function(RTFunction),
    Integer(i64),
}

impl Display for RTValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RTValue::Function(_) => f.write_str("<Function>"),
            RTValue::Integer(i) => f.write_fmt(format_args!("{}", *i)),
        }
    }
}

pub type RTFunction = Rc<dyn Fn(RTValue, &Globals) -> Result<RTValue, String>>;
