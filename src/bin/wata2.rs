use icfpc2022::*;
use once_cell::sync::Lazy;

pub static FLIP_ROTATE: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_ROTATE")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let mut png = read_png(&input);
    let mut best = (wata::INF, vec![]);
    for _ in 0..2 {
        for _ in 0..2 {
            let out = wata::solve2(&png);
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
    let mut canvas = Canvas::new(png.len(), png[0].len());
    eprintln!("move cost = {}", canvas.apply_all(best.1.clone()));
    eprintln!("diff cost = {}", similarity(&png, &canvas.bitmap));
    for p in best.1 {
        println!("{}", p);
    }
}
