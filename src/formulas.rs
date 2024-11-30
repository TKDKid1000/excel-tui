use std::process::id;

const OPERATORS: [&'static str; 18] = [
    "-", "%", "^", "^", "*", "/", "+", "&", "=", ">=", "<=", "<>", "<", ">", "@", "#", ":", ",",
];

#[derive(Debug, PartialEq, Clone, Default)]
pub enum TokenType {
    // TODO: Implement the commented out tokens
    //
    Function,
    // FunctionArg,
    // Reference,
    #[default] // I don't know if this is temporary or the actual default, but I'm too tired to
    // figure it out.
    Number,
    // String,
    // Boolean,
    Operator,
    LeftParen,
    RightParen,
    FuncLeftParen,
    FuncRightParen,
}

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
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
            Some(idx);
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
            });
            parse_idx -= 1;
        } else if OPERATORS.contains(&current_char.to_string().as_str()) {
            // Parse operators
            let next_char = formula.chars().nth(parse_idx).unwrap_or_default();
            let extended_operator = current_char.to_string() + next_char.to_string().as_str();

            // Check if it's >=, <=, or <>
            if OPERATORS.contains(&extended_operator.as_str()) {
                parsed.push(Token {
                    token_type: TokenType::Operator,
                    content: extended_operator,
                });
            } else {
                parsed.push(Token {
                    token_type: TokenType::Operator,
                    content: current_char.to_string(),
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
                        content: function_name,
                    });
                    parsed.push(Token {
                        token_type: TokenType::FuncLeftParen,
                        content: String::new(),
                    });
                    if let Some(close_paren_idx) = find_close_paren(formula, parse_idx) {
                        func_close_parens.push(close_paren_idx);
                    }

                    parse_idx -= 1;
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
            });
        } else if current_char == ')' {
            // Parse right parentheses
            if func_close_parens.contains(&parse_idx) {
                parsed.push(Token {
                    token_type: TokenType::FuncRightParen,
                    content: String::new(),
                });
            } else {
                parsed.push(Token {
                    token_type: TokenType::RightParen,
                    content: String::new(),
                });
            }
        }

        parse_idx += 1
    }

    for idx in 0..parsed.len() - 1 {
        if parsed[idx].token_type == TokenType::Operator
            && parsed[idx].content == "-"
            && (idx == 0 || parsed[idx - 1].token_type != TokenType::Number)
        // TODO: Number, or variable, or function
        {
            // Handle the special case of negation
            // https://math.stackexchange.com/questions/217315
            parsed[idx].content = String::from("-1");
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
        _ => a,
    }
}

pub fn eval_formula(formula: &str) -> Result<String, ()> {
    let parsed = parse_formula(formula).unwrap_or_default(); // TODO: Add some error checking

    // TODO: Support for non-numbers
    let mut output_queue: Vec<Token> = Vec::new();
    let mut operator_stack: Vec<Token> = Vec::new();

    for token in parsed[1..].iter() {
        // This is me skipping the = at the start
        match token.token_type {
            TokenType::LeftParen => {
                operator_stack.push(token.clone());
            }
            TokenType::RightParen => {
                // TODO: When if-let chains are implemented, make this an if-let expression
                while let Some(x) = operator_stack.pop() {
                    if x.token_type != TokenType::LeftParen {
                        output_queue.push(x);
                    } else {
                        break;
                    }
                }
            }
            TokenType::Operator => {
                let current_precedence = get_operator_precedence(token.content.as_str());
                println!("{}", current_precedence);

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
    }

    while operator_stack.len() > 0 {
        if let Some(popped) = operator_stack.pop() {
            output_queue.push(popped);
        }
    }

    let mut eval_stack: Vec<f32> = Vec::new();
    for token in output_queue.iter() {
        match token.token_type {
            TokenType::Operator => {
                // TODO: Add support for non-arithmetic operators
                let operator = token.content.as_str();
                let a = eval_stack.pop().unwrap();
                match operator {
                    "-1" => {
                        eval_stack.push(-a);
                    }
                    _ => {
                        let b = eval_stack.pop().unwrap();

                        eval_stack.push(apply_arithmetic_operator(b, a, operator));
                    }
                }
            }
            TokenType::Number => {
                eval_stack.push(token.content.parse().unwrap()); // TODO: Evil unwrap
            }
            _ => {
                // Ignore things like parentheses, which will no longer be with us.
            }
        }
    }

    println!(
        "Output: {:?}\nOperator: {:?}\nEval: {:?}",
        output_queue, operator_stack, eval_stack
    );

    Ok(String::new())
}
