use std::{collections::HashMap, f32::consts::PI, sync::OnceLock};

use crate::{
    formulas::{Token, TokenType},
    spreadsheet::Spreadsheet,
};

pub fn get_funcs() -> &'static HashMap<&'static str, &'static (dyn FormulaFunction + Sync)> {
    static FUNCTIONS: OnceLock<HashMap<&str, &(dyn FormulaFunction + Sync)>> = OnceLock::new();
    FUNCTIONS.get_or_init(|| {
        let mut m: HashMap<&str, &(dyn FormulaFunction + Sync)> = HashMap::new();
        m.insert("SUM", &Sum {});
        m.insert("SQRT", &Sqrt {});
        m.insert("IF", &If {});
        m.insert("PI", &Pi {});
        m.insert("RAND", &Rand {});
        m
    })
}

pub fn get_func(name: &str) -> Option<&&(dyn FormulaFunction + Sync)> {
    return get_funcs().get(name);
}

pub trait FormulaFunction {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()>;
}

struct Sum;
impl FormulaFunction for Sum {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()> {
        if !args.iter().all(|t| t.is_number(spreadsheet)) {
            return Err(());
        }
        Ok(vec![Token::new(
            TokenType::Number,
            args.iter()
                .map(|t| t.content.parse::<f32>().unwrap())
                .sum::<f32>()
                .to_string(),
        )])
    }
}

struct Sqrt;
impl FormulaFunction for Sqrt {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()> {
        if args.len() == 1 && args[0].token_type == TokenType::Number {
            return Ok(vec![Token::new(
                TokenType::Number,
                args[0].content.parse::<f32>().unwrap().sqrt().to_string(),
            )]);
        }
        return Err(());
    }
}

struct If;
impl FormulaFunction for If {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()> {
        // Fluffing if-let chaining again
        if args.len() < 2 {
            return Err(());
        }
        let condition = &args[0];

        if condition.token_type != TokenType::Boolean {
            return Err(());
        }

        if condition.as_f32(spreadsheet) == 1.0 {
            return Ok(vec![args[1].clone()]);
        } else {
            return Ok(vec![args
                .get(2)
                .unwrap_or(&Token::new(TokenType::Boolean, String::from("FALSE")))
                .clone()]);
        }
    }
}

struct Pi;
impl FormulaFunction for Pi {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()> {
        if args.len() > 0 {
            return Err(());
        }
        return Ok(vec![Token::new(TokenType::Number, PI.to_string())]);
    }
}

struct Rand;
impl FormulaFunction for Rand {
    fn call(&self, args: &[Token], spreadsheet: &Spreadsheet) -> Result<Vec<Token>, ()> {
        if args.len() > 0 {
            return Err(());
        }

        return Ok(vec![Token::new(
            TokenType::Number,
            rand::random::<f64>().to_string(),
        )]);
    }
}
