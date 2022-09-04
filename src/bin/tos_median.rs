use std::env::args;

use icfpc2022::{color::best_color2, read_png};

fn main() {
    let arg = args().nth(1).unwrap(); // "problems/16.png")
    let png = read_png(&arg);
    let (color, cost) = best_color2(&png, 0, 400, 0, 400);
    dbg!((color, cost));
}
