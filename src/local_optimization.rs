use super::{Color, Program};
use crate::{score, BlockId, Canvas, Move, WHITE};
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashSet;

const WIDTH: i32 = 400;

////////////////////////////////////////////////////////////////////////////////////////////////////
// 座標関連
////////////////////////////////////////////////////////////////////////////////////////////////////

/*
pub fn optimize_step(
    program: Program,
    image: &Vec<Vec<Color>>,
    diff_steps: &[i32],
) -> Option<(Program, f64)> {
    let original_score = Canvas::new400()
        .apply_all_and_score(program.clone(), image)
        .unwrap();

    for i in 0..program.len() {
        if let Move::LineCut(bid, ori, offset) = &program[i] {
            for d in diff_steps {
                if 0 < offset + *d && offset + *d < WIDTH {
                    let mut new_program = program.clone();
                    new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);

                    if let Ok(new_score) =
                        Canvas::new400().apply_all_and_score(new_program.clone(), image)
                    {
                        if new_score < original_score {
                            // eprintln!("Improve: {} -> {}", original_score, new_score);
                            return Some((new_program, new_score));
                        }
                    }
                }
            }
        }
    }

    None
}
 */

pub fn optimize_step_parallel(
    program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
    diff_steps: &[i32],
) -> Option<(Program, f64)> {
    let original_score = score(&program, initial_canvas, image).unwrap();

    let mut tmp = vec![];
    (0..program.len())
        .into_par_iter()
        .map(|i| {
            let mut result = None;
            if let Move::LineCut(bid, ori, offset) = &program[i] {
                for d in diff_steps {
                    if 0 < offset + *d && offset + *d < WIDTH {
                        let mut new_program = program.clone();
                        new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);
                        if let Ok(new_score) = score(&new_program, initial_canvas, image) {
                            if new_score < original_score {
                                //eprintln!("Improve: {} -> {}", original_score, new_score);
                                result = Some((new_program.clone(), new_score));

                                // ついでに色やる
                                let new_program2 =
                                    optimize_color(new_program.clone(), initial_canvas, image);
                                if let Ok(new_score2) = score(&new_program2, initial_canvas, image)
                                {
                                    // eprintln!(
                                    //     "Improve: {} -> {} -> {}",
                                    //     original_score, new_score, new_score2
                                    // );
                                    if new_score2 < new_score {
                                        result = Some((new_program2, new_score2));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            result
        })
        .collect_into_vec(&mut tmp);

    // for result in tmp {
    //     if result.is_some() {
    //         return result;
    //     }
    // }

    if let Some((best_program, best_score)) = tmp
        .into_iter()
        .filter_map(|option| option)
        .min_by_key(|(_, score)| ordered_float::OrderedFloat(*score))
    {
        // eprintln!("Color: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return Some((best_program, best_score));
        }
    }

    return None;
}

pub fn optimize_coord_two(
    program: &Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
    diff_step: i32,
    n_candidates: usize,
) -> Option<(Program, f64)> {
    let original_score = score(&program, initial_canvas, image).unwrap();

    let cut_steps: Vec<_> = program
        .iter()
        .enumerate()
        .filter_map(|(i, mov)| {
            if matches!(mov, Move::LineCut(_, _, _)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let mut modifications = vec![];
    for i in 0..cut_steps.len() {
        for j in 0..i {
            for si in [-1, 1] {
                for sj in [-1, 1] {
                    modifications.push((cut_steps[j], cut_steps[i], sj * diff_step, si * diff_step))
                }
            }
        }
    }
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([13; 32]);
    modifications.shuffle(&mut rng);

    let bar = indicatif::ProgressBar::new(modifications.len() as u64);
    bar.set_style(indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")
        .progress_chars("#>-"));

    let mut results = vec![];
    modifications
        .into_par_iter()
        .map(|(step1, step2, diff1, diff2)| {
            bar.inc(1);
            let mut new_program = program.clone();
            if let Move::LineCut(bid1, ori1, off1) = &program[step1] {
                if let Move::LineCut(bid2, ori2, off2) = &program[step2] {
                    new_program[step1] = Move::LineCut(bid1.clone(), *ori1, *off1 + diff1);
                    new_program[step2] = Move::LineCut(bid2.clone(), *ori2, *off2 + diff2);

                    if score(&new_program, initial_canvas, image).is_err() {
                        return None;
                    }
                    let new_program = optimize_color(new_program, initial_canvas, image);
                    let new_score = score(&new_program, initial_canvas, image).unwrap();
                    // dbg!(new_score);
                    return Some((new_program, new_score));
                } else {
                    panic!()
                }
                // new_program[step1]
            } else {
                panic!()
            }
            None
        })
        .collect_into_vec(&mut results);

    if let Some((best_program, best_score)) = results
        .into_iter()
        .filter_map(|option| option)
        .min_by_key(|(_, score)| ordered_float::OrderedFloat(*score))
    {
        eprintln!("Coord^2: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return Some((best_program, best_score));
        }
    }
    return None;

    /*
    let mut tmp = vec![];
    (0..program.len())
        .into_par_iter()
        .map(|i| {
            let mut result = None;
            if let Move::LineCut(bid, ori, offset) = &program[i] {
                for d in diff_steps {
                    if 0 < offset + *d && offset + *d < WIDTH {
                        let mut new_program = program.clone();
                        new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);
                        if let Ok(new_score) = score(&new_program, initial_canvas, image) {
                            if new_score < original_score {
                                //eprintln!("Improve: {} -> {}", original_score, new_score);
                                result = Some((new_program.clone(), new_score));

                                // ついでに色やる
                                let new_program2 =
                                    optimize_color(new_program.clone(), initial_canvas, image);
                                if let Ok(new_score2) = score(&new_program2, initial_canvas, image)
                                {
                                    // eprintln!(
                                    //     "Improve: {} -> {} -> {}",
                                    //     original_score, new_score, new_score2
                                    // );
                                    if new_score2 < new_score {
                                        result = Some((new_program2, new_score2));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            result
        })
        .collect_into_vec(&mut tmp);

    // for result in tmp {
    //     if result.is_some() {
    //         return result;
    //     }
    // }

    if let Some((best_program, best_score)) = tmp
        .into_iter()
        .filter_map(|option| option)
        .min_by_key(|(_, score)| ordered_float::OrderedFloat(*score))
    {
        // eprintln!("Color: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return Some((best_program, best_score));
        }
    }

    return None;
     */
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// 色関連
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn optimize_color(
    mut program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
) -> Program {
    // (1) まず、Colorを全てユニークにしていく。
    for (i, m) in program.iter_mut().enumerate() {
        if let Move::Color(_, c) = m {
            *c = (i as u32).to_le_bytes();
        }
    }

    // (2) 次に色を決めていく
    let mut canvas = initial_canvas.clone();
    canvas.bitmap.iter_mut().for_each(|row| row.fill(WHITE));
    canvas.apply_all(program.clone());

    // (3) 各Color命令が実際に塗るピクセルの色を集めていく
    let mut points = vec![vec![]; program.len()];
    for y in 0..canvas.bitmap.len() {
        for x in 0..canvas.bitmap[y].len() {
            let c = &canvas.bitmap[y][x];
            if c == &WHITE {
                continue;
            }
            let i = u32::from_le_bytes(*c) as usize;
            points[i].push(image[y][x]);
        }
    }

    // (4) 実際の色を決めてもどしていく
    for (i, m) in program.iter_mut().enumerate() {
        if let Move::Color(_, c) = m {
            let points_u8 = &points[i];
            if points_u8.is_empty() {
                // eprintln!("Color epmty: {}", i);
                continue;
            }
            let points_f64: Vec<_> = points_u8.iter().map(|cs| cs.map(|c| c as f64)).collect();

            let optimal_color = super::color::geometric_median_4d(&points_f64[..]);
            let (optimal_color, _) =
                super::color::round_to_optimal_u8_color(points_u8, &optimal_color);

            *c = optimal_color;
        }
    }

    program
}

pub fn try_removing_color_op(
    program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
) -> (Program, f64) {
    let original_score = score(&program, initial_canvas, image).unwrap();

    let mut tmp = vec![];
    (0..program.len())
        .into_par_iter()
        .map(|i| {
            if let Move::Color(_, _) = program[i] {
                let mut candidate_program = program.clone();
                candidate_program.remove(i);
                let candidate_program = optimize_color(candidate_program, initial_canvas, image);
                let candidate_score = score(&candidate_program, initial_canvas, image).unwrap();

                let candidate_program2 = remove_unnecessary_operations(&candidate_program);
                let candidate_score2 = score(&candidate_program2, initial_canvas, image).unwrap();
                assert!(candidate_score2 <= candidate_score);

                Some((candidate_score2, candidate_program2))
            } else {
                None
            }
        })
        .collect_into_vec(&mut tmp);

    if let Some((best_score, best_program)) = tmp
        .into_iter()
        .filter_map(|option| option)
        .min_by_key(|(score, _)| ordered_float::OrderedFloat(*score))
    {
        // eprintln!("Color: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return (best_program, best_score);
        }
    }
    return (program, original_score);
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// 不要なMoveの削除
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Color, Swapより後の不要なMoveを削除する。
/// もしremove_unnecessary_operationsがMergeを真面目にやったら不要になる。
pub fn trim_unnecessary_operations(mut program: Program) -> Program {
    while let Some(mov) = program.last() {
        if !mov.may_change_bitmap() {
            program.pop();
        } else {
            break;
        }
    }
    program
}

fn get_global_counter_history(program: &Program) -> Vec<u32> {
    let mut canvas = Canvas::new400();
    let mut history = vec![0; program.len()];
    for (i, mov) in program.iter().enumerate() {
        history[i] = canvas.counter;
        canvas.apply(mov);
    }
    history
}

/// merge専用。高々1個しか削除しないので、
pub fn remove_unnecessary_merge(program: &Program) -> Option<Program> {
    let mut used_block_ids = HashSet::new();
    let global_counter_history = get_global_counter_history(program);

    let mut step_to_remove = None;
    for (i, mov) in program.iter().enumerate().rev() {
        match mov {
            Move::Color(bid, _) | Move::LineCut(bid, _, _) | Move::PointCut(bid, _, _) => {
                used_block_ids.insert(bid.clone());
            }
            Move::Swap(bid0, bid1) => {
                used_block_ids.insert(bid0.clone());
                used_block_ids.insert(bid1.clone());
            }
            Move::Merge(bid0, bid1) => {
                used_block_ids.insert(bid0.clone());
                used_block_ids.insert(bid1.clone());

                let new_bid = BlockId(vec![global_counter_history[i] + 1]);
                if !used_block_ids.contains(&new_bid) {
                    step_to_remove = Some(i);
                    break;
                }
            }
        }
    }

    if step_to_remove.is_none() {
        return None;
    }
    let step_to_remove = step_to_remove.unwrap();
    dbg!(&step_to_remove);

    todo!()
}

pub fn remove_unnecessary_operations(program: &Program) -> Program {
    let program = trim_unnecessary_operations(program.clone());

    let mut used_block_ids = HashSet::new();
    let mut removable = vec![false; program.len()];

    for (i, mov) in program.iter().enumerate().rev() {
        match mov {
            Move::Color(bid, _) => {
                // Colorは、使うんだよね
                used_block_ids.insert(bid.clone());
            }
            Move::LineCut(bid, _, _) => {
                let used = bid.cut().iter().any(|b| used_block_ids.contains(&b));
                if used {
                    used_block_ids.insert(bid.clone());
                } else {
                    removable[i] = true;
                }
            }
            Move::PointCut(bid, _, _) => {
                let used = bid.cut4().iter().any(|b| used_block_ids.contains(&b));
                if used {
                    used_block_ids.insert(bid.clone());
                } else {
                    removable[i] = true;
                }
            }
            Move::Merge(bid0, bid1) => {
                // TODO: merge消すと全体の番号がずれて結構面倒なので後で
                //let new_bid = BlockId(vec![global_counter_history[i] + 1]);
                //if used_block_ids.contains(&new_bid) {
                if true {
                    used_block_ids.insert(bid0.clone());
                    used_block_ids.insert(bid1.clone());
                } else {
                    removable[i] = true;
                }
            }
            Move::Swap(bid0, bid1) => {
                // TODO: 一応、塗られた内容が同じなら不要ではある
                used_block_ids.insert(bid0.clone());
                used_block_ids.insert(bid1.clone());
            }
        }
    }

    let program = program
        .iter()
        .zip(removable)
        .filter_map(|(mov, rem)| {
            if rem {
                // println!("{}", mov);
                None
            } else {
                Some(mov.clone())
            }
        })
        .collect();

    // 余興
    // remove_unnecessary_merge(&program);

    program
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// メインループ
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn optimize(
    mut best_program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
    max_diff_step: i32,
    parallel: bool,
) -> (Program, f64) {
    let mut best_score = super::canvas::score(&best_program, initial_canvas, image).unwrap();
    if best_program.is_empty() {
        return (best_program, best_score);
    }

    let mut diff_step = 1;
    while diff_step <= max_diff_step {
        // Try color improvement
        if diff_step == 1 {
            let (new_program, new_score) =
                try_removing_color_op(best_program.clone(), initial_canvas, &image);
            if new_score < best_score {
                eprintln!("Improvement - Color: {} -> {}", best_score, new_score);
                best_program = new_program;
                best_score = new_score;
                continue;
            }
        }

        // Try coordinate improvement
        let ret = if parallel {
            optimize_step_parallel(
                best_program.clone(),
                initial_canvas,
                image,
                &[-diff_step, diff_step],
            )
        } else {
            todo!();
            //optimize_step(best_program.clone(), &image, &[-diff_step, diff_step])
        };
        if let Some((new_program, new_score)) = ret {
            if new_score < best_score {
                eprintln!("Improvement - Coord: {} -> {}", best_score, new_score);
                best_program = new_program;
                best_score = new_score;

                diff_step = 1;
                continue;
            }
        }

        eprintln!(
            "(Improvement failed, increasing step: {} -> {})",
            diff_step,
            diff_step + 1
        );
        diff_step += 1;
    }

    // TODO: 時間かかるならこれをループの中に入れる
    // icfpc2022::write_isl(
    //     std::fs::File::create(format!(
    //         "out/opt_{}_{:06.0}",
    //         sub.problem_id, improved_score
    //     ))
    //     .unwrap(),
    //     improved_program,
    // )
    // .unwrap();

    (best_program, best_score)
}
