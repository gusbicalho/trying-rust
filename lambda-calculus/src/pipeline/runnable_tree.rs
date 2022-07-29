use std::{cell::RefCell, collections::HashMap};

pub struct Names {
    interned: RefCell<HashMap<String, usize>>,
}

impl Names {
    pub fn new() -> Self {
        Names {
            interned: RefCell::new(HashMap::new()),
        }
    }
}
