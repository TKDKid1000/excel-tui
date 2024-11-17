use std::{collections::HashMap, hash::Hash};

trait Memoizable {
    type Args;
    type Result;

    fn call(&self, args: Self::Args) -> Self::Result;
}

struct Memoizer<F>
where
    F: Memoizable,
{
    cache: HashMap<F::Args, F::Result>,
    func: F,
}

impl<F> Memoizer<F>
where
    F: Memoizable,
    F::Args: Eq + Hash + Clone,
    F::Result: Clone,
{
    fn new(func: F) -> Self {
        Memoizer {
            cache: HashMap::new(),
            func,
        }
    }

    fn call(&mut self, args: F::Args) -> F::Result {
        if let Some(result) = self.cache.get(&args) {
            return result.clone();
        }
        let result = self.func.call(args.clone());
        self.cache.insert(args.clone(), result.clone());
        return result;
    }
}
