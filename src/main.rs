use std::env;

mod formulas;
mod ui;

fn main() {
    let _args: Vec<String> = env::args().collect();
    let _ = ui::start_ui();
    // let formula = //formulas::parse_formula("A5:B12,C5:D16").unwrap();
    // formulas::parse_formula("=SUM(--(COUNTIF(D5:D12,B5:B16, False)>=0))+\"Hello there\"+1+(A5:b26)")
    //     .unwrap();
    // println!("Parsed values:");
    // for part in formula.iter() {
    //     println!("{}", part)
    // }
}
