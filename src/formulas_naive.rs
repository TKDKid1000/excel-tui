use core::fmt;
use std::{cmp, vec};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const OPERATORS: [&'static str; 18] = [
    "-", "%", "^", "^", "*", "/", "+", "&", "=", ">=", "<=", "<>", "<", ">", "@", "#", ":", ",",
];

#[derive(Debug, PartialEq, Clone)]
pub enum FormulaPartType {
    Function,
    FunctionArg,
    Reference,
    Number,
    String,
    Boolean,
    Operator,
    Parent,
}

#[derive(Debug, Clone)]
pub struct FormulaPart {
    pub part_type: FormulaPartType,
    pub children: Vec<FormulaPart>,
    pub content: String,
}

impl FormulaPart {
    pub fn as_number(&self) -> Result<f32, ()> {
        if self.part_type != FormulaPartType::Number {
            return Err(());
        }
        if let Ok(parsed) = self.content.parse::<f32>() {
            return Ok(parsed);
        }
        return Err(());
    }
}

impl fmt::Display for FormulaPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.children.len() > 0 {
            let mut children_print = String::new();
            for part in self.children.iter() {
                children_print += part.to_string().as_str();
            }
            write!(
                f,
                "[{:#?} {}]: (\n\t{}\n)",
                self.part_type, self.content, children_print
            )
        } else {
            write!(f, "[{:#?} {}]", self.part_type, self.content)
        }
    }
}

pub fn find_close_paren(text: &str, opening: usize) -> Result<usize, ()> {
    let mut counter = 0;
    for i in opening..text.len() {
        let char = text.chars().nth(i).unwrap();
        if char == '(' {
            counter += 1;
        }
        if char == ')' {
            counter -= 1;
        }
        if counter == 0 {
            return Ok(i);
        }
    }
    Err(())
}

pub fn naive_parse_string(text: &str, opening: usize) -> Result<usize, ()> {
    // Does not yet account for escapes, which Excel may or may not even support.
    for i in opening + 1..text.len() {
        let char = text.chars().nth(i).unwrap();
        if char == '"' {
            return Ok(i);
        }
    }
    Err(())
}

pub fn parse_formula(formula: &str) -> Result<Vec<FormulaPart>, ()> {
    let mut parse_idx: usize = 0;
    if formula.starts_with("=") {
        parse_idx += 1;
    }

    let mut parsed: Vec<FormulaPart> = vec![];

    while parse_idx < formula.len() {
        let mut char = formula.chars().nth(parse_idx).unwrap();

        // Function parsing
        // All Excel function start with a letter...
        if char.is_ascii_alphabetic() {
            let function_start_idx = parse_idx; // Save this, so we can return to it if it's not a function.
            let mut function_name = String::new();
            // But they don't all contain only letters.
            while (char.is_ascii_alphanumeric() || char == '.') && parse_idx < formula.len() {
                char = formula.chars().nth(parse_idx).unwrap();
                function_name += char.to_string().as_str();
                parse_idx += 1;
            }
            if function_name.chars().last().unwrap() == '(' {
                // Begin function content parsing
                let close_paren = find_close_paren(formula, parse_idx - 1);
                if close_paren.is_err() {
                    return Err(());
                }

                let formula_slice = &formula[parse_idx..close_paren.unwrap()];
                let mut split_paren_depth = 0;
                let mut split_next_idx = 0;
                let function_args: Vec<&str> = formula_slice
                    .split(|c| {
                        split_next_idx += 1;
                        if c == '(' {
                            split_paren_depth += 1;
                        }
                        if c == ')' {
                            split_paren_depth -= 1;
                        }
                        let splitting = c == ','
                            && formula_slice.chars().nth(split_next_idx).unwrap_or('1') == ' '
                            && split_paren_depth == 0;
                        return splitting;
                    })
                    .collect();

                let mut function_args_parsed: Vec<FormulaPart> = vec![];
                for fn_arg in function_args.iter() {
                    let fn_arg_parsed = parse_formula(fn_arg);
                    if fn_arg_parsed.is_err() {
                        println!("Error on {}!", fn_arg);
                        return Err(());
                    }
                    function_args_parsed.push(FormulaPart {
                        part_type: FormulaPartType::FunctionArg,
                        children: fn_arg_parsed.unwrap(),
                        content: String::new(),
                    })
                }

                parsed.push(FormulaPart {
                    part_type: FormulaPartType::Function,
                    children: function_args_parsed,
                    content: function_name[..function_name.len() - 1].to_string(), // Remove the trailing parentheses
                });

                parse_idx = close_paren.unwrap();
            } else {
                parse_idx = function_start_idx;
            }
        }

        // Whenever the parse_idx is forceibly changed (quite often), this needs to be run:
        char = formula.chars().nth(parse_idx).unwrap();

        // Plain parenthesis parsing
        if char == '(' {
            let close_paren = find_close_paren(formula, parse_idx);
            if close_paren.is_err() {
                return Err(());
            }
            let paren_parsed = parse_formula(&formula[parse_idx + 1..close_paren.unwrap()]);
            if paren_parsed.is_err() {
                return Err(());
            }
            parsed.push(FormulaPart {
                part_type: FormulaPartType::Parent,
                children: paren_parsed.unwrap(),
                content: String::new(),
            });
            parse_idx = close_paren.unwrap();
        }

        // Whenever the parse_idx is forceibly changed (quite often), this needs to be run:
        // char = formula.chars().nth(parse_idx).unwrap();

        // Operator parsing
        for &operator in OPERATORS.iter() {
            if &formula[parse_idx..cmp::min(parse_idx + operator.len(), formula.len())] == operator
            {
                parsed.push(FormulaPart {
                    part_type: FormulaPartType::Operator,
                    children: vec![],
                    content: String::from(operator),
                });
                parse_idx = parse_idx + operator.len() - 1;
            }
        }

        // Whenever the parse_idx is forceibly changed (quite often), this needs to be run:
        char = formula.chars().nth(parse_idx).unwrap();

        // Numbers
        if char.is_ascii_digit() || char == '.' {
            let mut digits = String::new();
            // But they don't all contain only letters.
            // digits += char.to_string().as_str();

            while parse_idx < formula.len()
                && (formula.chars().nth(parse_idx).unwrap().is_ascii_digit()
                    || formula.chars().nth(parse_idx).unwrap() == '.')
            {
                char = formula.chars().nth(parse_idx).unwrap();
                digits += char.to_string().as_str();
                parse_idx += 1;
            }

            if digits.len() > 0 {
                parsed.push(FormulaPart {
                    part_type: FormulaPartType::Number,
                    children: vec![],
                    content: digits,
                });
                parse_idx -= 1;
            }
        }

        // Strings
        if char == '"' {
            let ending = naive_parse_string(formula, parse_idx).unwrap_or(formula.len() - 1);
            parsed.push(FormulaPart {
                part_type: FormulaPartType::String,
                children: vec![],
                content: formula[parse_idx + 1..ending].to_string(),
            });
            parse_idx = ending;
        }

        // Booleans
        if formula[parse_idx..cmp::min(parse_idx + "TRUE".len(), formula.len())]
            .to_ascii_uppercase()
            == "TRUE"
        {
            parse_idx = parse_idx + "TRUE".len();
            parsed.push(FormulaPart {
                part_type: FormulaPartType::Boolean,
                children: vec![],
                content: "TRUE".to_string(),
            })
        } else if formula[parse_idx..cmp::min(parse_idx + "FALSE".len(), formula.len())]
            .to_ascii_uppercase()
            == "FALSE"
        {
            parse_idx = parse_idx + "FALSE".len();
            parsed.push(FormulaPart {
                part_type: FormulaPartType::Boolean,
                children: vec![],
                content: "FALSE".to_string(),
            })
        }

        // References
        if char.is_ascii_alphabetic() {
            let start_idx = parse_idx; // Store this, so we can come back to it if it's not a reference.
            let mut reference = String::new();
            // reference += char.to_string().as_str();

            while parse_idx < formula.len()
                && formula
                    .chars()
                    .nth(parse_idx)
                    .unwrap()
                    .is_ascii_alphanumeric()
            {
                char = formula.chars().nth(parse_idx).unwrap();
                reference += char.to_string().as_str();
                parse_idx += 1;
            }

            if reference.chars().last().unwrap_or(' ').is_ascii_digit() {
                // All references should end with a number.
                parse_idx -= 1;

                parsed.push(FormulaPart {
                    part_type: FormulaPartType::Reference,
                    children: vec![],
                    content: reference.to_ascii_uppercase(),
                });
            } else {
                // Not a reference, go back.
                parse_idx = start_idx;
            }
        }

        parse_idx += 1
    }

    Ok(parsed)
}

fn apply_operator(a: f32, b: f32, operator: &str) -> f32 {
    match operator {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => a / b,
        _ => a,
    }
}

#[derive(EnumIter)]
enum OrderOpsState {
    Reference,
    Negation,
    Percent,
    Exponentiation,
    Multiplication,
    Addition,
    Concatenation,
    Comparison,
}

pub fn eval_formula(formula: &str) -> Result<String, ()> {
    let mut evaluated_formula: Vec<FormulaPart> = vec![];
    let mut call_stack: Vec<String> = vec![];

    if let Ok(parsed) = parse_formula(formula) {

        // for operation in OrderOpsState::iter() {
        //     match operation {
        //         OrderOpsState::Reference => {}
        //         OrderOpsState::Negation => (),
        //         OrderOpsState::Percent => (),
        //         OrderOpsState::Exponentiation => (),
        //         OrderOpsState::Multiplication => {
        //             handle_multiplication_operation(&mut evaluated_formula, &parsed).unwrap();
        //         }
        //         OrderOpsState::Addition => {
        //             handle_addition_operation(&mut evaluated_formula, &parsed).unwrap();
        //         }
        //         OrderOpsState::Concatenation => (),
        //         OrderOpsState::Comparison => (),
        //     }
        // }
    }

    Ok(evaluated_formula[0].content.clone())
}

fn handle_multiplication_operation(
    evaluated_formula: &mut Vec<FormulaPart>,
    parsed: &Vec<FormulaPart>,
) -> Result<Vec<FormulaPart>, ()> {
    let mut idx = 0;

    while idx < parsed.len() {
        let part = &parsed[idx];
        match part.part_type {
            FormulaPartType::Operator => {
                if idx == 0 || idx >= parsed.len() {
                    return Err(());
                }
                if part.content != "*" && part.content != "/" {
                    continue;
                }

                let prev = evaluated_formula.last().unwrap();
                let next = parsed[idx + 1].clone();

                let last_idx = evaluated_formula.len() - 1;

                if prev.part_type != FormulaPartType::String && prev.part_type == next.part_type {
                    // Logic for if they're the same (numbers and slices)
                    if prev.part_type == FormulaPartType::Number {
                        evaluated_formula[last_idx].content = apply_operator(
                            prev.as_number().unwrap(),
                            next.as_number().unwrap(),
                            part.content.as_str(),
                        )
                        .to_string();
                        idx += 1;
                    }
                }
                {
                    // Treat everything elsae like strings and concat away
                }
            }
            FormulaPartType::Number => {
                evaluated_formula.push(part.clone());
            }
            _ => (),
        }
        idx += 1;
    }
    Ok(evaluated_formula.clone())
}
fn handle_addition_operation(
    evaluated_formula: &mut Vec<FormulaPart>,
    parsed: &Vec<FormulaPart>,
) -> Result<Vec<FormulaPart>, ()> {
    let mut idx = 0;

    while idx < parsed.len() {
        let part = &parsed[idx];
        match part.part_type {
            FormulaPartType::Operator => {
                if idx == 0 || idx >= parsed.len() {
                    return Err(());
                }
                if part.content != "+" && part.content != "-" {
                    continue;
                }

                let prev = evaluated_formula.last().unwrap();
                let next = parsed[idx + 1].clone();

                let last_idx = evaluated_formula.len() - 1;

                if prev.part_type != FormulaPartType::String && prev.part_type == next.part_type {
                    // Logic for if they're the same (numbers and slices)
                    if prev.part_type == FormulaPartType::Number {
                        evaluated_formula[last_idx].content = apply_operator(
                            prev.as_number().unwrap(),
                            next.as_number().unwrap(),
                            part.content.as_str(),
                        )
                        .to_string();
                        idx += 1;
                    }
                }
                {
                    // Treat everything else like strings and concat away
                }
            }
            FormulaPartType::Number => {
                evaluated_formula.push(part.clone());
            }
            _ => (),
        }
        idx += 1;
    }
    Ok(evaluated_formula.clone())
}
