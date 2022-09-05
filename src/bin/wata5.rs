use icfpc2022::{
    submissions::{find_best_score, find_best_submission, read_submission_program},
    wata::{get_swapped_png, merge_solution, MAX_CANDIDATES, MAX_WIDTH},
    *,
};
use once_cell::sync::Lazy;

pub static FLIP_X: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_X")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

pub static FLIP_Y: Lazy<i32> = Lazy::new(|| {
    std::env::var("FLIP_Y")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

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

pub static SWAP: Lazy<i32> = Lazy::new(|| {
    std::env::var("SWAP")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

pub static SWAP_BORDER: Lazy<f64> = Lazy::new(|| {
    std::env::var("SWAP_BORDER")
        .unwrap_or("20.0".to_owned())
        .parse()
        .unwrap()
});

pub static SWAP_COMBO: Lazy<usize> = Lazy::new(|| {
    std::env::var("SWAP_COMBO")
        .unwrap_or("8".to_owned())
        .parse()
        .unwrap()
});

pub static SWAP_FILE: Lazy<String> =
    Lazy::new(|| std::env::var("SWAP_FILE").unwrap_or(String::new()));

fn main() {
    let problem_id = std::env::args().nth(1).unwrap().parse::<u32>().unwrap();
    let (init_canvas, mut png) = load_problem(problem_id);
    let orig_png = png.clone();
    let mut best = (wata::INF, vec![]);

    let best_flips;
    if *FLIP_ROTATE_BEST_ONLY != 0 {
        best_flips = Some(submissions::find_best_flip(problem_id).unwrap());
        eprintln!(
            "Best flip only mode ON, best flips: x={} y={}",
            best_flips.unwrap().0,
            best_flips.unwrap().1
        );
    } else if *FLIP_X + *FLIP_Y > 0 {
        best_flips = Some((*FLIP_X > 0, *FLIP_Y > 0));
    } else {
        best_flips = None;
    }

    let swap = if SWAP_FILE.len() > 0 {
        Some(read_isl(std::fs::File::open(&*SWAP_FILE).unwrap()).unwrap())
    } else if *SWAP == 2 {
        let best = read_submission_program(find_best_submission(problem_id).unwrap().id)
            .unwrap()
            .1;
        Some(best)
    } else {
        None
    };

    let swap = if let Some(swap) = swap {
        let mut canvas = init_canvas.clone();
        let mut start = 0;
        let mut ok = false;
        for p in 0..swap.len() {
            if canvas.blocks.len() == 1 {
                start = p;
            }
            if let Move::Swap(_, _) = &swap[p] {
                ok = true;
                break;
            }
            canvas.apply(&swap[p]);
        }
        if ok {
            let mut canvas = init_canvas.clone();
            canvas.apply_all(swap[0..start].iter().cloned());
            let id = canvas.counter;
            let mut swap2 = vec![];
            for p in swap[start..].iter() {
                let mut p = p.clone();
                p.inc_id(!id + 1);
                swap2.push(p);
            }
            png = get_swapped_png(&png, &swap2, &init_canvas);
            Some(swap2)
        } else {
            None
        }
    } else {
        None
    };

    let mut flip_x = false;
    let mut flip_y = false;
    for _ in 0..2 {
        for _ in 0..2 {
            if best_flips.map_or(true, |b| b == (flip_x, flip_y)) {
                let out = if *SWAP == 1 {
                    let out = chokudai1::solve_swap2(
                        &mut png.clone(),
                        *SWAP_BORDER,
                        *SWAP_COMBO,
                        &init_canvas,
                        &|png, init_canvas| wata::solve5(png, init_canvas, true),
                    )
                    .1;
                    (
                        init_canvas
                            .clone()
                            .apply_all_and_score(out.clone(), &png)
                            .unwrap(),
                        out,
                    )
                } else if swap.is_some() {
                    wata::solve5(&png, &init_canvas, true)
                } else {
                    wata::solve5(&png, &init_canvas, false)
                };
                if best.0.setmin(out.0) {
                    eprintln!("{}", best.0);
                    best.1 = out.1;
                }
            }

            if best_flips.is_none() && *FLIP_ROTATE == 0 {
                break;
            }
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &init_canvas);
            png = rotate::rotate_png(&png);
            flip_x = !flip_x;
            flip_y = !flip_y;
        }
        if best_flips.is_none() && *FLIP_ROTATE == 0 {
            break;
        }
        best.1 = rotate::flip_program_with_initial_canvas(&best.1, &init_canvas);
        png = rotate::flip_png(png);
        flip_x = !flip_x;
    }
    eprintln!("{}", best.0);
    if let Some(swap) = &swap {
        best.1 = merge_solution(&init_canvas, &best.1, swap);
    }
    let mut canvas = init_canvas;
    let move_score = canvas.apply_all(best.1.clone());
    let diff_score = similarity(&orig_png, &canvas.bitmap);
    best.0 = move_score + diff_score;
    eprintln!("best = {}", best.0);
    eprintln!("move cost = {}", move_score);
    eprintln!("diff cost = {}", diff_score);
    let best_score = find_best_score(problem_id);
    if best_score > best.0.round() as u32 {
        eprintln!(
            "!!!!!!!!!!!!!!!!!!!! improved !!!!!!!!!!!!!!!!!!!!\n{} -> {}",
            best_score,
            best.0.round()
        );
    }
    println!(
        "# wata5 FLIP_ROTATE={} FLIP_ROTATE_BEST_ONLY={} MAX_WIDTH={} MAX_CANDIDATES={} SWAP={}",
        *FLIP_ROTATE, *FLIP_ROTATE_BEST_ONLY, *MAX_WIDTH, *MAX_CANDIDATES, *SWAP,
    );
    if let Some(swap) = swap {
        for p in swap {
            println!("# {}", p);
        }
    }
    for p in best.1 {
        println!("{}", p);
    }
}
