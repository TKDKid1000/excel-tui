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

pub trait StringPadding {
    fn left_pad(&self, length: usize, pad_char: char) -> String;
    fn right_pad(&self, length: usize, pad_char: char) -> String;
    fn center(&self, length: usize, pad_char: char) -> String;
}

impl StringPadding for String {
    fn left_pad(&self, length: usize, pad_char: char) -> String {
        if self.len() >= length {
            return self.clone();
        }
        let mut working = self.clone();
        while working.len() < length {
            working.insert(0, pad_char);
        }
        working
    }

    fn right_pad(&self, length: usize, pad_char: char) -> String {
        if self.len() >= length {
            return self.clone();
        }
        let mut working = self.clone();
        while working.len() < length {
            working.push(pad_char);
        }
        working
    }

    fn center(&self, length: usize, pad_char: char) -> String {
        if self.len() >= length {
            return self.clone();
        }
        let mut working = self.clone();
        while working.len() < length {
            // Alternate adding to the start and the end
            if working.len() % 2 == 0 {
                working.insert(0, pad_char);
            } else {
                working.push(pad_char);
            }
        }
        working
    }
}
