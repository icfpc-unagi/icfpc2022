use icfpc2022;
use icfpc2022::chokudai1::solve_swap;
use icfpc2022::chokudai_dev2::*;
use icfpc2022::read_png;
use icfpc2022::*;
//use std::collections::HashMap;

fn main() {
    let input = std::env::args().nth(1).unwrap();

    let mut png = read_png(&input);
    let mut best = (99999999.9, vec![]);

    for _i1 in 0..1 {
        for _i2 in 0..4 {
            let mut png2 = png.clone();
            let out = solve_swap(&mut png2, 10.0, 5);
            let mut canvas = icfpc2022::Canvas::new400();
            let score = canvas.apply_all_and_score(out.1.clone(), &png).unwrap();

            eprintln!("score: {}", score);
            if best.0.setmin(score) {
                eprintln!("bestscore!: {}", best.0);
                best.1 = out.1;
            }
            best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
        }
        //best.1 = rotate::flip_program(&best.1);
        //png = rotate::flip_png(png);
    }
    eprintln!("{}", best.0);
    let mut canvas = Canvas::new(png.len(), png[0].len());
    eprintln!("best cost = {}", best.0);
    eprintln!(
        "cost = {}",
        canvas.apply_all_and_score(best.1.clone(), &png).unwrap()
    );
    //eprintln!("move cost = {}", canvas.apply_all(best.1.clone()));
    //eprintln!("diff cost = {}", similarity(&png, &canvas.bitmap));

    println!("# chokudai monte with swap");
    for p in best.1 {
        println!("{}", p);
    }
}
