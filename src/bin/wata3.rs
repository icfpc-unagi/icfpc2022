use icfpc2022::*;
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
    let problem_id = std::env::args().nth(1).unwrap();
    // let mut png = read_png(&format!("problems/{}.png", problem_id));
    // let init_canvas =
    //     if std::path::Path::new(&format!("problems/{}.initial.json", problem_id)).exists() {
    //         Canvas::from_initial_json(&format!("problems/{}.initial.json", problem_id))
    //     } else {
    //         Canvas::new(png[0].len(), png.len())
    //     };
    let (init_canvas, mut png) = load_problem(problem_id.parse::<u32>().unwrap());
    let mut best = (wata::INF, vec![]);

    let best_flips;
    if *FLIP_ROTATE_BEST_ONLY != 0 {
        best_flips = Some(submissions::find_best_flip(problem_id.parse().unwrap()).unwrap());
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
            best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
            best.1 = rotate::rotate_program(&best.1);
            png = rotate::rotate_png(&png);
            flip_x = !flip_x;
            flip_y = !flip_y;
        }
        if *FLIP_ROTATE_BEST_ONLY == 0 && *FLIP_ROTATE == 0 {
            break;
        }
        best.1 = rotate::flip_program(&best.1);
        png = rotate::flip_png(png);
        flip_x = !flip_x;
    }
    eprintln!("{}", best.0);
    let mut canvas = init_canvas;
    eprintln!("move cost = {}", canvas.apply_all(best.1.clone()));
    eprintln!("diff cost = {}", similarity(&png, &canvas.bitmap));
    for p in best.1 {
        println!("{}", p);
    }
}
