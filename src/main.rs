#![allow(unused)]
use std::env;
use std::io::Result;

use clap::Parser;
use config::Config;
use formulas::eval_formula;
use spreadsheet::Spreadsheet;

mod app;
mod config;
mod formula_functions;
mod formulas;
mod references;
mod spreadsheet;
mod ui;
mod undo_stack;
mod utils;

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "PATH", help = "Path to a CSV or XLSX file.")]
    path: Option<String>,

    #[arg(
        short,
        long,
        required = false,
        help = "Evaluate a formula and write its result to stdout."
    )]
    formula: Option<String>,

    #[arg(
        short,
        long,
        action,
        help = "Replace Nerd Font icons with plain text representations."
    )]
    ascii: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let spreadsheet = if let Some(path) = args.path {
        Spreadsheet::from_csv(&path)?
    } else {
        Spreadsheet::new()
    };

    if let Some(formula) = args.formula {
        println!("{}", eval_formula(&formula, &spreadsheet).unwrap().content);
        return Ok(());
    }

    let mut terminal = app::init()?;
    let mut app = app::App::new(Config {
        nerd_font: !args.ascii,
    });
    app.spreadsheet = spreadsheet;

    let app_result = app.run(&mut terminal);
    app::restore()?;
    app_result
}
