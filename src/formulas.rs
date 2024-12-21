use std::collections::BTreeSet;

use crate::formula_functions::{get_func, get_funcs};
use crate::references::{parse_reference, Reference};
use crate::spreadsheet::Spreadsheet;

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
    Reference,
    Number,
    #[default] // String is definitely the default
    String,
    Boolean,
    Operator,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub function_n_args: Option<u8>,
    pub reference_set: Option<BTreeSet<Reference>>,
}

impl Token {
    pub fn new(token_type: TokenType, content: String) -> Token {
        Token {
            token_type,
            content,
            function_n_args: None,
            reference_set: None,
        }
    }

    pub fn function(content: String, n_args: u8) -> Token {
        Token {
            token_type: TokenType::Function,
            content: content,
            function_n_args: Some(n_args),
            reference_set: None,
        }
    }

    pub fn reference(refs: BTreeSet<Reference>) -> Token {
        Token {
            token_type: TokenType::Reference,
            content: String::new(),
            function_n_args: None,
            reference_set: Some(refs),
        }
    }

    pub fn as_f32(&self, spreadsheet: &Spreadsheet) -> f32 {
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

    pub fn is_number(&self, spreadsheet: &Spreadsheet) -> bool {
        match self.token_type {
            TokenType::Boolean => true,
            TokenType::Number => true,
            TokenType::String => self.content.parse::<f32>().is_ok(),
            // TODO: Handle multi-refs
            TokenType::Reference => {
                self.content.split(",").all(|r| {
                    if let Some(reference) = parse_reference(r) {
                        if let Ok(cell_value) = spreadsheet.get_cell_value(&reference.get_cell()) {
                            return cell_value.is_number(spreadsheet);
                        }
                    }
                    false
                })
                // spreadsheet.get_cell_value()
            }
            _ => false,
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

            parsed.push(Token::new(TokenType::Number, number_content));
            parse_idx -= 1;
        } else if OPERATORS.contains(&current_char.to_string().as_str()) {
            // Parse operators
            let next_char = formula.chars().nth(parse_idx + 1).unwrap_or_default();
            let extended_operator = current_char.to_string() + next_char.to_string().as_str();

            // Check if it's >=, <=, or <>
            if OPERATORS.contains(&extended_operator.as_str()) {
                parsed.push(Token::new(TokenType::Operator, extended_operator));
                parse_idx += 1 // Increment parse index because we consumed the next char
            } else {
                parsed.push(Token::new(TokenType::Operator, current_char.to_string()));
            }
        } else if current_char.is_ascii_alphabetic() {
            // Parse functions, booleans, and (most) cell references.

            let mut textual_content = String::new();
            // Allow for multiple numerical characters to follow one another, as is usual
            while formula
                .chars()
                .nth(parse_idx)
                .unwrap_or_default()
                .is_ascii_alphanumeric()
            {
                textual_content += formula
                    .chars()
                    .nth(parse_idx)
                    .unwrap_or_default()
                    .to_string()
                    .as_str();
                parse_idx += 1;
            }

            if textual_content.to_uppercase() == "TRUE" || textual_content.to_uppercase() == "FALSE"
            {
                parsed.push(Token::new(
                    TokenType::Boolean,
                    textual_content.to_uppercase(),
                ));
                // Decrement parse index because it went over by one in the while loop.
                parse_idx -= 1
            } else if get_funcs().contains_key(textual_content.to_uppercase().as_str()) {
                // Checks if the text is a valid function name, in which case then it proceeds with function parsing.

                // TODO: Again, chain if-let statements...
                if let Some(func_open_paren) = formula.chars().nth(parse_idx) {
                    if func_open_paren == '(' {
                        parsed.push(Token::function(textual_content.to_uppercase(), 0));
                        if let Some(close_paren_idx) = find_close_paren(formula, parse_idx) {
                            func_close_parens.push(close_paren_idx);
                        }

                        // Resetting with parse_idx -= 1 should NOT happen because the left parenthesis should be consumed
                    } else {
                        eprintln!("Error: Function doesn't have an opening parenthesis");
                        return Err(()); // Function doesn't have an opening parenthesis
                    }
                }
            } else if let Some(parsed_ref) = parse_reference(&textual_content.to_uppercase()) {
                // Only need to know if it's successful, not the resulting ref
                parsed.push(Token::reference(BTreeSet::from([parsed_ref])));
                // Decrement parse index because it went over by one in the while loop.
                parse_idx -= 1
            }
        } else if current_char == '(' {
            // Parse left parentheses

            parsed.push(Token::new(TokenType::LeftParen, String::new()));
        } else if current_char == ')' {
            // Parse right parentheses

            if func_close_parens.contains(&parse_idx) {
                parsed.push(Token::new(TokenType::FuncClose, String::new()));
            } else {
                parsed.push(Token::new(TokenType::RightParen, String::new()));
            }
        } else if current_char == '"' {
            // Parse string
            let mut string_value = String::new();

            // TODO: if-while (if-let for searchability) chaining... man I need this
            parse_idx += 1;
            while let Some(char) = formula.chars().nth(parse_idx) {
                // TODO: Alter this condition to allow Excel's frankly weird "" escaping
                if char == '"' {
                    break;
                }
                string_value += &char.to_string();
                parse_idx += 1;
            }

            parsed.push(Token::new(TokenType::String, string_value));
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

    // Remove "to remove" elements
    for idx in (0..parsed.len()).rev() {
        if to_remove.contains(&idx) {
            parsed.remove(idx);
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
        ":" => 9, // This needs a higher precedence than is listed on Excel's website
        "," => 8,
        " " => 8,
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

fn apply_reference_operator(
    a: BTreeSet<Reference>,
    b: BTreeSet<Reference>,
    operator: &str,
) -> BTreeSet<Reference> {
    BTreeSet::from_iter(
        match operator {
            ":" => a.first().unwrap().range(b.first().unwrap()),
            "," => a.union(&b).cloned().collect::<Vec<Reference>>(),
            " " => a.intersection(&b).cloned().collect::<Vec<Reference>>(),
            _ => a.iter().cloned().collect::<Vec<Reference>>(),
        }
        .into_iter(),
    )
}

pub fn cell_to_token(cell_value: &str, spreadsheet: &Spreadsheet) -> Result<Token, ()> {
    // Parses a single cell as a single value (boolean or number), else a string
    // Unless, of course, it's another formula-
    if cell_value.starts_with("=") {
        return eval_formula(&cell_value[1..], spreadsheet);
    }
    let mut token_type = TokenType::Number;
    if cell_value.chars().all(|c| c.is_ascii_digit() || c == '.') {
        token_type = TokenType::Number;
    } else if cell_value.to_uppercase() == "FALSE" || cell_value.to_uppercase() == "True" {
        token_type = TokenType::Boolean;
    }
    Ok(Token::new(token_type, cell_value.to_string()))
}

pub fn eval_formula(formula: &str, spreadsheet: &Spreadsheet) -> Result<Token, ()> {
    let parsed = parse_formula(formula).unwrap_or_default(); // TODO: Add some error checking

    // TODO: Support for non-numbers
    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();
    let mut function_stack: Vec<Token> = Vec::new();

    for token in parsed.iter() {
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
                output_queue.push(function_stack.pop().unwrap());
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
            TokenType::String | TokenType::Boolean | TokenType::Number | TokenType::Reference => {
                output_queue.push(token.clone());
            }
        }
    }

    while operator_stack.len() > 0 {
        if let Some(popped) = operator_stack.pop() {
            output_queue.push(popped);
        }
    }

    let mut eval_stack: Vec<Token> = Vec::new();
    for token in output_queue.iter() {
        match token.token_type {
            TokenType::Operator => {
                // TODO: Add support for non-arithmetic operators
                let operator = token.content.as_str();
                let a = eval_stack.pop().unwrap();
                match operator {
                    ":" | "," | " " => {
                        let b = eval_stack.pop().unwrap();
                        if !(a.token_type == TokenType::Reference
                            && b.token_type == TokenType::Reference)
                        {
                            eprintln!("Reference operation error");
                            return Err(());
                        }
                        eval_stack.push(Token::reference(apply_reference_operator(
                            a.reference_set.unwrap(),
                            b.reference_set.unwrap(),
                            operator,
                        )));
                    }
                    "-1" => {
                        eval_stack.push(Token::new(
                            TokenType::Number,
                            (-a.as_f32(spreadsheet)).to_string(),
                        ));
                    }
                    "%" => {
                        eval_stack.push(Token::new(
                            TokenType::Number,
                            (a.as_f32(spreadsheet) / 100.).to_string(),
                        ));
                    }
                    "+" | "-" | "*" | "/" | "^" => {
                        let b = eval_stack.pop().unwrap();

                        eval_stack.push(Token::new(
                            TokenType::Number,
                            apply_arithmetic_operator(
                                b.as_f32(spreadsheet),
                                a.as_f32(spreadsheet),
                                operator,
                            )
                            .to_string(),
                        ));
                    }
                    "&" => {
                        let b = eval_stack.pop().unwrap();

                        let mut concatenated = b.content + a.content.as_str();

                        // Determine type of concatenated variable (it may be a string, number, or boolean)
                        let mut concatenated_type = TokenType::String;
                        if concatenated.parse::<f32>().is_ok() {
                            concatenated_type = TokenType::Number
                        } else if concatenated.to_uppercase() == "TRUE"
                            || concatenated.to_uppercase() == "FALSE"
                        {
                            concatenated_type = TokenType::Boolean;
                            concatenated = concatenated.to_uppercase();
                        }

                        eval_stack.push(Token::new(concatenated_type, concatenated));
                    }
                    "=" | "<" | ">" | "<=" | ">=" | "<>" => {
                        let b: Token = eval_stack.pop().unwrap();

                        eval_stack.push(Token::new(
                            TokenType::Boolean,
                            apply_comparison_operator(
                                b.as_f32(spreadsheet),
                                a.as_f32(spreadsheet),
                                operator,
                            )
                            .to_string()
                            .to_uppercase(),
                        ));
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
                    if let Ok(result) = func.call(args.as_slice(), spreadsheet) {
                        // println!("Result of function {}: {:?}", token.content, result);
                        eval_stack.extend(result);
                    }
                } else {
                    return Err(());
                }
            }
            TokenType::Reference => {
                // TODO: Handle lists of references
                // if let Some(refs) = &token.reference_set {
                //     if refs.len() == 1 {
                //         let reference = refs.first().unwrap(); // Safe unwrap :)
                //         let value = spreadsheet.get_cell_value(&reference.get_cell()).unwrap();
                //         eval_stack.push(value)
                //         // TODO: Evil unwrap
                //     } else {
                //         eval_stack.push(token.clone());
                //     }
                // } else {
                //     return Err(());
                // }
                eval_stack.push(token.clone());
            }
            TokenType::String | TokenType::Boolean | TokenType::Number => {
                eval_stack.push(token.clone());
            }
            _ => {
                // Ignore things like parentheses, which will no longer be with us.
            }
        }
    }

    // TODO: Allow returning multiple things for those oddly specific functions
    Ok(eval_stack.first().unwrap().clone())
    // Ok(eval_stack.first().unwrap().content.to_string()) // TODO: Don't return just a String
}
