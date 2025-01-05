use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
    iter::zip,
};

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

pub trait LevenshteinDistance {
    fn levenshtein(self, other: &str) -> usize;
}

impl LevenshteinDistance for String {
    fn levenshtein(self, other: &str) -> usize {
        let mut matrix = vec![vec![0; self.len()]; other.len()]; // Declare self.len() x other.len()
                                                                 // matrix of zeroes
        for i in 1..self.len() {
            matrix[0][i] = i;
        }

        for j in 1..other.len() {
            matrix[j][0] = j
        }

        for j in 1..other.len() {
            for i in 1..self.len() {
                let subs_cost = if self.chars().nth(i) == other.chars().nth(j) {
                    0
                } else {
                    1
                };

                matrix[j][i] = *[
                    matrix[j][i - 1] + 1,
                    matrix[j - 1][i] + 1,
                    matrix[j - 1][i - 1] + subs_cost,
                ]
                .iter()
                .min()
                .unwrap();
            }
        }

        return matrix[other.len() - 1][self.len() - 1];
    }
}

pub trait FuzzySearch {
    fn fuzzy_search(self, search: &str, max_distance: usize) -> Vec<String>;
}

impl FuzzySearch for Vec<String> {
    fn fuzzy_search(self, search: &str, max_distance: usize) -> Vec<String> {
        // Uses a similar matching system to VSCode, where it returns strings that contain
        // characters in the order of the search, sorting by the amount that are at the start.

        let mut scores: Vec<i16> = Vec::new();
        for test_str in self.iter() {
            let mut search_idx = 0;
            let mut score = 0; // Lower score is better.
            if test_str.len() == 0 {
                // Never match empty strings.
                scores.push(-1);
                continue;
            }
            for tc in test_str.chars() {
                if tc == search.chars().nth(search_idx).unwrap() {
                    search_idx += 1;
                    if search_idx == search.len() {
                        scores.push(score);
                        break;
                    }
                } else {
                    score += 1;
                }
            }
            if search_idx != search.len() {
                scores.push(-1); // -1 means failure, which will be filtered out.
            }
        }
        let mut scores_map = zip(scores, self.clone())
            .filter(|(score, _)| *score >= 0 && *score <= max_distance as i16)
            .collect::<Vec<(i16, String)>>();
        scores_map.sort_by_key(|s| s.0);
        scores_map
            .iter()
            .map(|s| s.1.clone())
            .collect::<Vec<String>>()
    }
}
