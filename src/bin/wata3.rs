use icfpc2022::{submissions::find_best_score, *};
use once_cell::sync::Lazy;

pub static FLIP_ROTATE: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_ROTATE")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

pub static FLIP_ROTATE_BEST_ONLY: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_ROTATE_BEST_ONLY")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

fn main() {
    let problem_id = std::env::args().nth(1).unwrap().parse::<u32>().unwrap();
    let (init_canvas, mut png) = load_problem(problem_id);
    let mut best = (wata::INF, vec![]);

    let best_flips;
    if *FLIP_ROTATE_BEST_ONLY != 0 {
        best_flips = Some(submissions::find_best_flip(problem_id).unwrap());
        eprintln!(
            "Best flip only mode ON, best flips: x={} y={}",
            best_flips.unwrap().0,
            best_flips.unwrap().1
        );
    } else {
        best_flips = None;
    }

    let mut flip_x = false;
    let mut flip_y = false;
    for _ in 0..2 {
        for _ in 0..2 {
            if best_flips.map_or(true, |b| b == (flip_x, flip_y)) {
                let out = wata::solve3(&png, &init_canvas);
                if best.0.setmin(out.0) {
                    eprintln!("{}", best.0);
                    best.1 = out.1;
                }
            }

            if *FLIP_ROTATE_BEST_ONLY == 0 && *FLIP_ROTATE == 0 {
                break;
            }
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            flip_x = !flip_x;
            flip_y = !flip_y;
        }
        if *FLIP_ROTATE_BEST_ONLY == 0 && *FLIP_ROTATE == 0 {
            break;
        }
        best.1 = rotate::flip_program_with_initial_canvas(&best.1, &init_canvas);
        png = rotate::flip_png(png);
        flip_x = !flip_x;
    }
    eprintln!("{}", best.0);
    let mut canvas = init_canvas;
    eprintln!("move cost = {}", canvas.apply_all(best.1.clone()));
    eprintln!("diff cost = {}", similarity(&png, &canvas.bitmap));
    let best_score = find_best_score(problem_id);
    if best_score > best.0.round() as u32 {
        eprintln!(
            "!!!!!!!!!!!!!!!!!!!! improved !!!!!!!!!!!!!!!!!!!!\n{} -> {}",
            best_score,
            best.0.round()
        );
    }
    for p in best.1 {
        println!("{}", p);
    }
}
