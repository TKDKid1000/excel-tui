use std::env;
use std::io::Result;

use formulas::eval_formula;
use spreadsheet::Spreadsheet;

mod app;
mod formula_functions;
mod formulas;
mod references;
mod spreadsheet;
mod ui;
mod undo_stack;
mod utils;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut spreadsheet = Spreadsheet::new();
    if args.len() >= 2 {
        spreadsheet = Spreadsheet::from_csv(&args[1]).unwrap();
    }
    if let Some(func_flag_idx) = args.iter().position(|a| a == "-f") {
        if args.len() > func_flag_idx + 1 {
            let formula = &args[func_flag_idx + 1];

            println!("{}", eval_formula(formula, &spreadsheet).unwrap().content);
            return Ok(());
        }
    }

    let mut terminal = app::init()?;
    let mut app = app::App::default();
    app.spreadsheet = spreadsheet;

    let app_result = app.run(&mut terminal);
    app::restore()?;
    app_result

    // let formula = //formulas::parse_formula("A5:B12,C5:D16").unwrap();
    // formulas::parse_formula("=SUM(--(COUNTIF(D5:D12,B5:B16, False)>=0))+\"Hello there\"+1+(A5:b26)")
    //     .unwrap();
    // let formula = "-3+-4*(2+(-2+3)*4)/5";
    // let formula = "6*(2+2)";
    // println!("Parsed values:");
    // for part in parse_formula(formula).unwrap() {
    //     println!("{:?}", part)
    // }
}
