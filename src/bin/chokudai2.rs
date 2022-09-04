use icfpc2022::{
    //chokudai_dev2::monte_solve2,
    chokudai1::solve_swap2,
    submissions::find_best_score,
    wata::{MAX_AREA, MAX_WIDTH},
    *,
};
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

    //TODO 引数で時間が弄れるようにする
    let sec = 300;

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
                let mut png2 = png.clone();
                let mut out = solve_swap2(&mut png2, sec, 20.0, 8, &init_canvas);

                //for mm in &out.1 {
                //    println!("{}", mm);
                //}

                out.0 = init_canvas
                    .clone()
                    .apply_all_and_score(out.1.clone(), &png)
                    .unwrap();

                eprintln!("realScore : {}", out.0);
                if best.0.setmin(out.0) {
                    eprintln!("best: {}", best.0);
                    best.1 = out.1;
                }
            }

            if *FLIP_ROTATE_BEST_ONLY == 0 && *FLIP_ROTATE == 100 {
                break;
            }
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            flip_x = !flip_x;
            flip_y = !flip_y;
        }
        if *FLIP_ROTATE_BEST_ONLY == 0 && *FLIP_ROTATE == 100 {
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
    println!("# chokudai-swap 300sec");

    for p in best.1 {
        println!("{}", p);
    }
}
