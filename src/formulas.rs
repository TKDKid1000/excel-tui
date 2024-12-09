use std::process::id;

use crate::formula_functions::get_func;

const OPERATORS: [&'static str; 19] = [
    "-", "%", "^", "^", "*", "/", "+", "&", "=", ">=", "<=", "<>", "<", ">", "@", "#", ":", ",",
    " ",
];

#[derive(Debug, PartialEq, Clone, Default)]
pub enum TokenType {
    // TODO: Implement the commented out tokens
    //
    Function,
    FuncClose,
    FuncArgSep, // aka a comma
    // FunctionArg,
    Reference, // TODO: Implement this (it's only uncommented for FuncArgSep logic)
    #[default] // I don't know if this is temporary or the actual default, but I'm too tired to
    // figure it out.
    Number,
    // String,
    Boolean, // TODO: Also implement this (only added for IF command)
    Operator,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub function_n_args: Option<u8>,
}

impl Token {
    pub fn as_f32(&self) -> f32 {
        match self.token_type {
            TokenType::Number => self.content.parse::<f32>().unwrap(),
            TokenType::Boolean => {
                if self.content == String::from("TRUE") {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

pub fn find_close_paren(formula: &str, start_idx: usize) -> Option<usize> {
    let mut paren_depth = 0;
    for idx in start_idx..formula.len() {
        match formula.chars().nth(idx).unwrap_or_default() {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => (),
        }
        if paren_depth == 0 {
            return Some(idx);
        }
    }
    None
}

pub fn parse_formula(formula: &str) -> Result<Vec<Token>, ()> {
    let mut parsed: Vec<Token> = Vec::new();
    let mut func_close_parens: Vec<usize> = Vec::new();

    let mut parse_idx = 0;
    while parse_idx < formula.len() {
        let current_char = formula.chars().nth(parse_idx).unwrap_or_default();
        if current_char.is_ascii_digit() {
            // Parse raw numbers
            let mut number_content = String::new();
            // Allow for multiple numerical characters to follow one another, as is usual
            while formula
                .chars()
                .nth(parse_idx)
                .unwrap_or_default()
                .is_ascii_digit()
            {
                number_content += formula
                    .chars()
                    .nth(parse_idx)
                    .unwrap_or_default()
                    .to_string()
                    .as_str();
                parse_idx += 1;
            }

            parsed.push(Token {
                token_type: TokenType::Number,
                content: number_content,
                function_n_args: None,
            });
            parse_idx -= 1;
        } else if OPERATORS.contains(&current_char.to_string().as_str()) {
            // Parse operators
            let next_char = formula.chars().nth(parse_idx + 1).unwrap_or_default();
            let extended_operator = current_char.to_string() + next_char.to_string().as_str();

            // Check if it's >=, <=, or <>
            if OPERATORS.contains(&extended_operator.as_str()) {
                parsed.push(Token {
                    token_type: TokenType::Operator,
                    content: extended_operator,
                    function_n_args: None,
                });
                parse_idx += 1 // Increment parse index because we consumed the next char
            } else {
                parsed.push(Token {
                    token_type: TokenType::Operator,
                    content: current_char.to_string(),
                    function_n_args: None,
                });
            }
        } else if current_char.is_ascii_alphabetic() {
            // Parse functions
            let mut function_name = String::new();
            // Allow for multiple numerical characters to follow one another, as is usual
            while formula
                .chars()
                .nth(parse_idx)
                .unwrap_or_default()
                .is_ascii_alphanumeric()
            {
                function_name += formula
                    .chars()
                    .nth(parse_idx)
                    .unwrap_or_default()
                    .to_string()
                    .as_str();
                parse_idx += 1;
            }

            // TODO: Again, chain if-let statements...
            if let Some(func_open_paren) = formula.chars().nth(parse_idx) {
                if func_open_paren == '(' {
                    parsed.push(Token {
                        token_type: TokenType::Function,
                        content: function_name.to_uppercase(),
                        function_n_args: None,
                    });
                    if let Some(close_paren_idx) = find_close_paren(formula, parse_idx) {
                        func_close_parens.push(close_paren_idx);
                    }

                    // Resetting with parse_idx -= 1 should NOT happen because the left parenthesis should be consumed
                } else {
                    return Err(()); // Function doesn't have an opening parenthesis
                }
            } else {
                return Err(()); // Function doesn't have an opening parenthesis
            }
        } else if current_char == '(' {
            // Parse left parentheses
            parsed.push(Token {
                token_type: TokenType::LeftParen,
                content: String::new(),
                function_n_args: None,
            });
        } else if current_char == ')' {
            // Parse right parentheses
            if func_close_parens.contains(&parse_idx) {
                parsed.push(Token {
                    token_type: TokenType::FuncClose,
                    content: String::new(),
                    function_n_args: None,
                });
            } else {
                parsed.push(Token {
                    token_type: TokenType::RightParen,
                    content: String::new(),
                    function_n_args: None,
                });
            }
        }

        parse_idx += 1
    }

    // Handle special cases of dual-meaning operators ("-" and "," and " ")
    let mut to_remove: Vec<usize> = Vec::new();
    for idx in 0..parsed.len() - 1 {
        if parsed[idx].token_type == TokenType::Operator
            && parsed[idx].content == "-"
            && (idx == 0 || parsed[idx - 1].token_type != TokenType::Number)
        // TODO: Number, or variable, or function
        {
            // Handle the special case of negation
            // https://math.stackexchange.com/questions/217315
            parsed[idx].content = String::from("-1");
        } else if parsed[idx].token_type == TokenType::Operator && parsed[idx].content == "," {
            if idx == 0 {
                // A comma can never be the first token, the last is ignored in the for loop
                return Err(());
            }
            if !(parsed[idx - 1].token_type == TokenType::Reference
                && parsed[idx + 1].token_type == TokenType::Reference)
            {
                parsed[idx].token_type = TokenType::FuncArgSep
            }
        } else if parsed[idx].token_type == TokenType::Operator && parsed[idx].content == " " {
            if idx == 0 {
                to_remove.push(idx);
            }
            if !(parsed[idx - 1].token_type == TokenType::Reference
                && parsed[idx + 1].token_type == TokenType::Reference)
            {
                to_remove.push(idx);
            }
        }
    }

    // Set function_n_args parameter for functions
    for idx in 0..parsed.len() {
        if parsed[idx].token_type == TokenType::Function {
            let mut function_depth = 0;
            let mut args = 1;

            for function_idx in idx..parsed.len() {
                if parsed[function_idx].token_type == TokenType::Function {
                    function_depth += 1;
                }
                if parsed[function_idx].token_type == TokenType::FuncClose {
                    function_depth -= 1;
                }

                if function_depth == 0 && function_idx == idx + 1 {
                    // Special case where the function opens and immediately closes.
                    args = 0;
                    break;
                }
                if function_depth == 0 {
                    break;
                }

                if function_depth == 1 && parsed[function_idx].token_type == TokenType::FuncArgSep {
                    // Reached a comma, and it's in the root function, not a nested one.
                    args += 1;
                }
            }

            parsed[idx].function_n_args = Some(args);
        }
    }

    return Ok(parsed);
}

fn get_operator_precedence(operator: &str) -> u8 {
    // TODO: Add the rest of these from Excel's docs
    match operator {
        // Reference operators
        ":" => 8,
        "," => 8,
        // Negation
        "-1" => 7,
        // Percent
        "%" => 6,
        // Exponentation
        "^" => 5,
        // Multiplication and division
        "*" => 4,
        "/" => 4,
        // Addition and subtraction
        "+" => 3,
        "-" => 3,
        // Concatenation
        "&" => 2,
        // Comparison
        "=" => 1,
        "<" => 1,
        ">" => 1,
        "<=" => 1,
        ">=" => 1,
        "<>" => 1,
        _ => 0,
    }
}

fn apply_arithmetic_operator(a: f32, b: f32, operator: &str) -> f32 {
    match operator {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => a / b,
        "^" => a.powf(b),
        _ => a,
    }
}

fn apply_comparison_operator(a: f32, b: f32, operator: &str) -> bool {
    match operator {
        "=" => a == b,
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        ">=" => a >= b,
        "<>" => a != b,
        _ => false,
    }
}

pub fn eval_formula(formula: &str) -> Result<String, ()> {
    let parsed = parse_formula(formula).unwrap_or_default(); // TODO: Add some error checking

    // TODO: Support for non-numbers
    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut function_stack: Vec<Token> = Vec::new();

    for token in parsed[1..].iter() {
        // This is me skipping the = at the start
        match token.token_type {
            TokenType::LeftParen => {
                operator_stack.push(token.clone());
            }
            TokenType::RightParen => {
                // TODO: When if-let chains are implemented, make this an if-let expression
                while let Some(x) = operator_stack.pop() {
                    if x.token_type != TokenType::LeftParen || x.token_type != TokenType::Function {
                        output_queue.push(x);
                    } else {
                        break;
                    }
                }
            }
            TokenType::Function => {
                function_stack.push(token.clone());
            }
            TokenType::FuncClose => {
                output_queue.push(function_stack.pop().unwrap());
                // while let Some(x) = operator_stack.pop() {
                //     if x.token_type != TokenType::LeftParen
                //         || x.token_type != TokenType::Function
                //         || x.token_type != TokenType::FuncArgSep
                //     {
                //         output_queue.push(x);
                //     } else {
                //         break;
                //     }
                // }
            }
            TokenType::FuncArgSep => {
                while let Some(x) = operator_stack.pop() {
                    if x.token_type != TokenType::LeftParen
                        || x.token_type != TokenType::Function
                        || x.token_type != TokenType::FuncArgSep
                    {
                        output_queue.push(x);
                    } else {
                        break;
                    }
                }
            }
            TokenType::Reference => {
                todo!()
            }
            TokenType::Boolean => {
                todo!()
            }
            TokenType::Operator => {
                let current_precedence = get_operator_precedence(token.content.as_str());

                // Okay to use unwrap_or here because any empty string will have a precedence of 1
                while get_operator_precedence(
                    &operator_stack
                        .last()
                        .unwrap_or(&Token::default())
                        .content
                        .as_str(),
                ) >= current_precedence
                {
                    if let Some(popped) = operator_stack.pop() {
                        output_queue.push(popped);
                    }
                }

                operator_stack.push(token.clone());
            }
            TokenType::Number => {
                output_queue.push(token.clone());
            }
        }
        // println!(
        //     "RPN: {:?}",
        //     output_queue
        //         .clone()
        //         .iter()
        //         .map(|t| &t.content)
        //         .collect::<Vec<&String>>()
        // );
    }

    while operator_stack.len() > 0 {
        if let Some(popped) = operator_stack.pop() {
            output_queue.push(popped);
        }
        // println!(
        //     "RPN: {:?}",
        //     output_queue
        //         .clone()
        //         .iter()
        //         .map(|t| &t.content)
        //         .collect::<Vec<&String>>()
        // );
    }

    // println!("Output Queue: {:?}", output_queue);

    let mut eval_stack: Vec<Token> = Vec::new();
    for token in output_queue.iter() {
        // println!("Eval Stack: {:?}", eval_stack);
        match token.token_type {
            TokenType::Operator => {
                // TODO: Add support for non-arithmetic operators
                let operator = token.content.as_str();
                let a = eval_stack.pop().unwrap();
                match operator {
                    "-1" => {
                        eval_stack.push(Token {
                            token_type: TokenType::Number,
                            content: (-a.as_f32()).to_string(),
                            function_n_args: None,
                        });
                    }
                    "%" => {
                        eval_stack.push(Token {
                            token_type: TokenType::Number,
                            content: (a.as_f32() / 100.).to_string(),
                            function_n_args: None,
                        });
                    }
                    "+" | "-" | "*" | "/" | "^" => {
                        let b = eval_stack.pop().unwrap();

                        eval_stack.push(Token {
                            token_type: TokenType::Number,
                            content: apply_arithmetic_operator(b.as_f32(), a.as_f32(), operator)
                                .to_string(),
                            function_n_args: None,
                        });
                    }
                    "=" | "<" | ">" | "<=" | ">=" | "<>" => {
                        let b: Token = eval_stack.pop().unwrap();

                        eval_stack.push(Token {
                            token_type: TokenType::Boolean,
                            content: apply_comparison_operator(b.as_f32(), a.as_f32(), operator)
                                .to_string()
                                .to_uppercase(),
                            function_n_args: None,
                        });
                    }
                    _ => {}
                }
            }
            TokenType::Function => {
                if let Some(func) = get_func(&token.content) {
                    // println!("Eval stack at {}: {:?}", &token.content, eval_stack)
                    let mut args = Vec::new();
                    for _ in 0..token.function_n_args.unwrap() {
                        args.push(eval_stack.pop().unwrap());
                    }
                    args.reverse(); // Makes writing the functions a hell of a lot easier
                    if let Ok(result) = func.call(args.as_slice()) {
                        // println!("Result of function {}: {:?}", token.content, result);
                        eval_stack.extend(result);
                    }
                } else {
                    return Err(());
                }
            }
            TokenType::Number => {
                eval_stack.push(token.clone()); // TODO: Evil unwrap
            }
            _ => {
                // Ignore things like parentheses, which will no longer be with us.
            }
        }
    }

    // println!("Eval: {:?}", eval_stack);

    Ok(eval_stack.first().unwrap().content.to_string()) // TODO: Don't return just a String
}
