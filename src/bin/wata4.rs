use icfpc2022::*;
use once_cell::sync::Lazy;

pub static FLIP_ROTATE: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_ROTATE")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

fn main() {
    let problem_id = std::env::args().nth(1).unwrap();
    let (init_canvas, mut png) = load_problem(problem_id.parse::<u32>().unwrap());
    let mut best = (wata::INF, vec![]);
    for _ in 0..2 {
        for _ in 0..2 {
            let out = wata::solve4(&png, &init_canvas);
            if best.0.setmin(out.0) {
                eprintln!("{}", best.0);
                best.1 = out.1;
            }
            if *FLIP_ROTATE == 0 {
                break;
            }
            best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
            best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
        }
        if *FLIP_ROTATE == 0 {
            break;
        }
        best.1 = rotate::flip_program(&best.1);
        png = rotate::flip_png(png);
    }
    eprintln!("{}", best.0);
    let mut canvas = init_canvas;
    eprintln!("move cost = {}", canvas.apply_all(best.1.clone()));
    eprintln!("diff cost = {}", similarity(&png, &canvas.bitmap));
    for p in best.1 {
        println!("{}", p);
    }
}
