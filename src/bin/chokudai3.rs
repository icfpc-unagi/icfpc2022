use icfpc2022::{
    local_optimization::optimize,
    optmerge::merge_all,
    submissions::find_best_score,
    wata::{get_swapped_png, merge_solution},
    *,
};
use once_cell::sync::Lazy;

pub static SWAP: Lazy<i32> = Lazy::new(|| {
    std::env::var("SWAP")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

pub static SWAP_FILE: Lazy<String> =
    Lazy::new(|| std::env::var("SWAP_FILE").unwrap_or(String::new()));

pub static TIMEOUT: Lazy<i32> = Lazy::new(|| {
    std::env::var("TIMEOUT")
        .unwrap_or("120".to_owned())
        .parse()
        .unwrap()
});

pub static TIME2: Lazy<i32> = Lazy::new(|| {
    std::env::var("TIME2")
        .unwrap_or("10".to_owned())
        .parse()
        .unwrap()
});

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
        .unwrap_or("1".to_owned())
        .parse()
        .unwrap()
});

fn main() {
    let problem_id = std::env::args().nth(1).unwrap().parse::<u32>().unwrap();
    let (init_canvas, mut png) = load_problem(problem_id);
    let orig_png = png.clone();
    let mut best = (wata::INF, vec![]);

    let sec1 = *TIMEOUT;
    let sec2 = *TIME2;

    let mut canvas2 = init_canvas.clone();
    let mv = merge_all(&mut canvas2);
    //let init_canvas = canvas2;
    let mut test_canvas = Canvas::new400();
    test_canvas.cost_type = init_canvas.cost_type;

    let swap = if SWAP_FILE.len() > 0 {
        let swap = read_isl(std::fs::File::open(&*SWAP_FILE).unwrap()).unwrap();
        let mut canvas = init_canvas.clone();
        let mut start = 0;
        for p in 0..swap.len() {
            if canvas.blocks.len() == 1 {
                start = p;
            }
            if let Move::Swap(_, _) = &swap[p] {
                break;
            }
            canvas.apply(&swap[p]);
        }
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
    };

    let mut flip_x = false;
    let mut flip_y = false;

    let mut bestflip_x = false;
    let mut bestflip_y = false;

    for t in 0..2 {
        let sec3 = {
            if t == 0 {
                sec2
            } else {
                sec1
            }
        };
        for _ in 0..2 {
            for _ in 0..2 {
                if t == 0 || (flip_x == bestflip_x && flip_y == bestflip_y) {
                    let out = if *SWAP > 0 {
                        let out = chokudai1::solve_swap2(
                            &mut png.clone(),
                            20.0,
                            8,
                            &test_canvas,
                            &|png, test_canvas| chokudai_dev2::monte_solve2(png, sec3, test_canvas),
                        )
                        .1;
                        (
                            test_canvas
                                .clone()
                                .apply_all_and_score(out.clone(), &png)
                                .unwrap(),
                            out,
                        )
                    } else {
                        let out = chokudai_dev2::monte_solve2(&png, sec3, &test_canvas).1;
                        (
                            test_canvas
                                .clone()
                                .apply_all_and_score(out.clone(), &png)
                                .unwrap(),
                            out,
                        )
                    };
                    if best.0.setmin(out.0) {
                        bestflip_x = flip_x;
                        bestflip_y = flip_y;
                        best.1 = out.1;
                        eprintln!("{} {} {} {}", best.0, bestflip_x, bestflip_y, t);
                    }
                }

                best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &test_canvas);
                png = rotate::rotate_png(&png);
                best.1 = rotate::rotate_program_with_initial_canvas(&best.1, &test_canvas);
                png = rotate::rotate_png(&png);
                flip_x = !flip_x;
                flip_y = !flip_y;
            }

            best.1 = rotate::flip_program_with_initial_canvas(&best.1, &test_canvas);
            png = rotate::flip_png(png);
            flip_x = !flip_x;
        }
    }
    eprintln!("{}", best.0);
    {
        best.1 = merge_solution(&init_canvas, &mv, &best.1);
    }
    if let Some(swap) = &swap {
        best.1 = merge_solution(&init_canvas, &best.1, swap);
    }
    let mut canvas = init_canvas.clone();
    let move_score = canvas.apply_all(best.1.clone());
    let diff_score = similarity(&orig_png, &canvas.bitmap);
    best.0 = move_score + diff_score;
    eprintln!("best score {} = {}", problem_id, best.0);
    eprintln!("move cost = {}", move_score);
    eprintln!("diff cost = {}", diff_score);

    let r = optimize(best.1, &init_canvas.clone(), &orig_png, 8, true, 0);
    eprintln!("final score {} = {}", problem_id, r.1);
    best.1 = r.0;
    /*
    let best_score = find_best_score(problem_id);
    if best_score > best.0.round() as u32 {
        eprintln!(
            "!!!!!!!!!!!!!!!!!!!! improved !!!!!!!!!!!!!!!!!!!!\n{} -> {}",
            best_score,
            best.0.round()
        );
    }
    */
    println!("# chokudai3");
    if let Some(swap) = swap {
        for p in swap {
            println!("# {}", p);
        }
    }
    for p in best.1 {
        println!("{}", p);
    }
}
