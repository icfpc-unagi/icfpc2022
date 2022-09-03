use icfpc2022::chokudai1::solve_swap;
use icfpc2022::read_png;
use icfpc2022::{BlockId, Move};
//use std::collections::HashMap;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let mut png = read_png(&input);
    let (score, ans) = solve_swap(&mut png, 3.0, 10);
    for p in ans {
        println!("{}", p);
    }
    eprintln!("{}", score);
}
