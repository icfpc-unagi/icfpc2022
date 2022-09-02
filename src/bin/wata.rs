use icfpc2022::*;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let mut png = read_png(&input);
    let mut best = (wata::INF, vec![]);
    for _ in 0..2 {
        for _ in 0..4 {
            let out = wata::solve(&png);
            if best.0.setmin(out.0) {
                eprintln!("{}", best.0);
                best.1 = out.1;
            }
            // best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
        }
        // best.1 = rotate::flip_program(&best.1);
        png = rotate::flip_png(png);
    }
    for p in best.1 {
        println!("{}", p);
    }
}
