use std::{collections::HashMap, sync::OnceLock};

use crate::formulas::TokenType;

pub fn get_func(name: &str) -> Option<&&(dyn FormulaFunction + Sync)> {
    static FUNCTIONS: OnceLock<HashMap<&str, &(dyn FormulaFunction + Sync)>> = OnceLock::new();
    FUNCTIONS
        .get_or_init(|| {
            let mut m: HashMap<&str, &(dyn FormulaFunction + Sync)> = HashMap::new();
            m.insert("SUM", &Sum {});
            m.insert("SQRT", &Sqrt {});
            m
        })
        .get(name)
}

pub trait FormulaFunction {
    fn call(&self, args: &[f32]) -> Result<Vec<f32>, ()>;
}

struct Sum;
impl FormulaFunction for Sum {
    fn call(&self, args: &[f32]) -> Result<Vec<f32>, ()> {
        // if !args.iter().all(|t| t.token_type == TokenType::Number) {
        //     return Err(());
        // }
        // Ok(vec![f32 {
        //     token_type: TokenType::Number,
        //     content: args
        //         .iter()
        //         .map(|t| t.content.parse::<f32>().unwrap())
        //         .sum::<f32>()
        //         .to_string(),
        // }])
        return Ok(vec![args.iter().sum()]);
    }
}

struct Sqrt;
impl FormulaFunction for Sqrt {
    fn call(&self, args: &[f32]) -> Result<Vec<f32>, ()> {
        if args.len() == 1
        /* && args[0].token_type == TokenType::Number */
        {
            // return Ok(vec![f32 {
            //     token_type: TokenType::Number,
            //     content: args[0].content.parse::<f32>().unwrap().sqrt().to_string(),
            // }]);
            return Ok(vec![args[0].sqrt()]);
        }
        return Err(());
    }
}
