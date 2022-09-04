use super::{Color, Program, Submission};
use crate::{Canvas, Move, WHITE};
use rayon::prelude::*;
use std::collections::HashSet;

const WIDTH: i32 = 400;

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
                            eprintln!("Improve: {} -> {}", original_score, new_score);
                            return Some((new_program, new_score));
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn optimize_step_parallel(
    program: Program,
    image: &Vec<Vec<Color>>,
    diff_steps: &[i32],
) -> Option<(Program, f64)> {
    let original_score = Canvas::new400()
        .apply_all_and_score(program.clone(), image)
        .unwrap();

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
                        if let Ok(new_score) =
                            Canvas::new400().apply_all_and_score(new_program.clone(), image)
                        {
                            if new_score < original_score {
                                eprintln!("Improve: {} -> {}", original_score, new_score);
                                result = Some((new_program.clone(), new_score));

                                // ついでに色やる
                                let new_program2 = optimize_color(new_program.clone(), image);
                                if let Ok(new_score2) = Canvas::new400()
                                    .apply_all_and_score(new_program2.clone(), image)
                                {
                                    eprintln!(
                                        "Improve: {} -> {} -> {}",
                                        original_score, new_score, new_score2
                                    );
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

    for result in tmp {
        if result.is_some() {
            return result;
        }
    }
    return None;
}

pub fn optimize(
    mut program: Program,
    image: &Vec<Vec<Color>>,
    max_diff_step: i32,
    parallel: bool,
) -> (Program, f64) {
    let mut result = (
        program.clone(),
        Canvas::new400()
            .apply_all_and_score(program.clone(), image)
            .unwrap(),
    );

    let mut diff_step = 1;
    while diff_step <= max_diff_step {
        let ret = if parallel {
            optimize_step_parallel(program.clone(), &image, &[-diff_step, diff_step])
        } else {
            optimize_step(program.clone(), &image, &[-diff_step, diff_step])
        };

        if let Some((improved_program, improved_score)) = ret {
            program = improved_program.clone();
            result = (improved_program, improved_score);

            // TODO: 時間かかるならこれ
            // icfpc2022::write_isl(
            //     std::fs::File::create(format!(
            //         "out/opt_{}_{:06.0}",
            //         sub.problem_id, improved_score
            //     ))
            //     .unwrap(),
            //     improved_program,
            // )
            // .unwrap();

            diff_step = 1;
        } else {
            eprintln!("Step: {} -> {}", diff_step, diff_step + 1);
            diff_step += 1;
        }
    }

    result
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn optimize_color(mut program: Program, image: &Vec<Vec<Color>>) -> Program {
    // (1) まず、Colorを全てユニークにしていく。
    for (i, m) in program.iter_mut().enumerate() {
        if let Move::Color(_, c) = m {
            *c = [
                (i % 256) as u8,
                (i / 256 % 256) as u8,
                (i / 256 / 256 % 256) as u8,
                (i / 256 / 256 / 256) as u8,
            ];
        }
    }

    // (2) 次に色を決めていく
    let mut canvas = Canvas::new400();
    // TODO: initial canvas対応した際にここ気をつけること！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！
    canvas.bitmap.iter_mut().for_each(|row| row.fill(WHITE));
    canvas.apply_all(program.clone());

    // (3) 各Color命令が実際に塗るピクセルの色を集めていく
    let mut points = vec![vec![]; program.len()];
    for y in 0..canvas.bitmap.len() {
        for x in 0..canvas.bitmap[y].len() {
            let c = &canvas.bitmap[y][x];
            // TODO: ここも！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！！
            if c == &WHITE {
                continue;
            }
            let i = c[0] as usize
                + (c[1] as usize + (c[2] as usize + (c[3] as usize) * 256) * 256) * 256;
            points[i].push(image[y][x]);
        }
    }

    // (4) 実際の色を決めてもどしていく
    for (i, m) in program.iter_mut().enumerate() {
        if let Move::Color(_, c) = m {
            let points_u8 = &points[i];
            if points_u8.is_empty() {
                eprintln!("Color epmty: {}", i);
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

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn try_removing_color_op(program: Program, image: &Vec<Vec<Color>>) -> (Program, f64) {
    let mut tmp = vec![];
    (0..program.len())
        .into_par_iter()
        .map(|i| {
            if let Move::Color(_, _) = program[i] {
                let mut candidate_program = program.clone();
                candidate_program.remove(i);
                let candidate_program = optimize_color(candidate_program, image);
                // TODO: remove unnecessary cut
                let candidate_score = super::canvas::score(&candidate_program, image).unwrap();
                Some((candidate_score, candidate_program))
            } else {
                None
            }
        })
        .collect_into_vec(&mut tmp);

    let original_score = super::canvas::score(&program, image).unwrap();
    if let Some((best_score, best_program)) = tmp
        .into_iter()
        .filter_map(|option| option)
        .min_by_key(|(score, _)| ordered_float::OrderedFloat(*score))
    {
        eprintln!("Color: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return (best_program, best_score);
        }
    }
    return (program, original_score);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// pub fn remove_unused_cuts(program: Program) -> Program {
//     let used_block_ids = HashSet::new();
//     let mut removable = vec![false; program.len()];
//
//     for mov in program.iter().enumerate().rev() {
//         match mov {
//             Move::LineCut(block_id, _, _) | Move::PointCut(block_id, _, _) | Move::Color(=> {}
//         }
//     }
//     todo!()
// }

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn read_submission(
    submission_id: u32,
) -> anyhow::Result<(Submission, Program, Vec<Vec<Color>>)> {
    let sub: Submission = serde_json::from_reader(std::fs::File::open(format!(
        "submissions/{}.json",
        submission_id
    ))?)?;
    assert_eq!(sub.status, "SUCCEEDED");
    let program = crate::read_isl(std::fs::File::open(format!(
        "submissions/{}.isl",
        submission_id
    ))?)?;
    let png = crate::read_png(&format!("problems/{}.png", sub.problem_id));

    Ok((sub, program, png))
}
