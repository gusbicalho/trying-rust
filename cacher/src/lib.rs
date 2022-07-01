use std::collections::HashMap;
use std::hash::Hash;

pub struct Cacher<Arg, Res, F> {
    f: F,
    cache: HashMap<Arg, Res>,
}

impl<Arg, Res, F> Cacher<Arg, Res, F> {
    pub fn new(f: F) -> Cacher<Arg, Res, F> {
        Cacher {
            f,
            cache: HashMap::new(),
        }
    }
}

impl<Arg, Res, F> Cacher<Arg, Res, F>
where
    F: Fn(&Arg) -> Res,
    Res: Copy,
    Arg: Hash + Eq,
{
    pub fn call(&mut self, arg: Arg) -> Res {
        *(self
            .cache
            .entry(arg)
            .or_insert_with_key(|key| (self.f)(key)))
    }
}

impl<Arg, Res, F> Cacher<Arg, Res, F>
where
    F: FnMut(&Arg) -> Res,
    Res: Copy,
    Arg: Hash + Eq,
{
    pub fn call_mut(&mut self, arg: Arg) -> Res {
        *(self
            .cache
            .entry(arg)
            .or_insert_with_key(|key| (self.f)(key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut calls = Vec::new();
        let mut cacher = Cacher::new(|v: &u32| {
            calls.push(*v);
            v + 2
        });
        assert_eq!(cacher.call_mut(2), 4);
        assert_eq!(cacher.call_mut(3), 5);
        assert_eq!(cacher.call_mut(2), 4);
        assert_eq!(cacher.call_mut(3), 5);
        assert_eq!(calls, vec![2, 3]);
    }
}
