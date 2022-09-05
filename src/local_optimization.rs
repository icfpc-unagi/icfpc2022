use super::{Color, Program};
use crate::{score, Block, BlockId, Canvas, Move, WHITE};
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

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

pub fn optimize_step_parallel_old(
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

pub fn optimize_step_parallel(
    program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
    diff_steps: &[i32],
) -> Option<(Program, f64)> {
    let original_score = score(&program, initial_canvas, image).unwrap();

    let ret = (0..program.len()).into_par_iter().find_map_any(|i| {
        if let Move::LineCut(bid, ori, offset) = &program[i] {
            for d in diff_steps {
                if 0 < offset + *d && offset + *d < WIDTH {
                    let mut new_program = program.clone();
                    new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);
                    if score(&new_program, initial_canvas, image).is_err() {
                        return None;
                    }
                    let new_program = optimize_color(new_program.clone(), initial_canvas, image);
                    let new_score = score(&new_program, initial_canvas, image);
                    if new_score.is_err() {
                        return None;
                    }
                    let new_score = new_score.unwrap();
                    if new_score < original_score {
                        return Some((new_program, new_score));
                    }
                }
            }
        }
        None
    });

    ret
}

pub fn optimize_coord_two(
    program: &Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
    diff_step: i32,
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

    // let mut results = vec![];
    let ret = modifications
        .into_par_iter()
        .find_map_any(|(step1, step2, diff1, diff2)| {
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

                    if new_score < original_score {
                        return Some((new_program, new_score));
                    } else {
                        return None;
                    }
                } else {
                    panic!()
                }
                // new_program[step1]
            } else {
                panic!()
            }
        });

    if let Some((best_program, best_score)) = ret {
        eprintln!("Coord^2: {} -> {}", original_score, best_score);
        if best_score < original_score {
            return Some((best_program, best_score));
        }
    }
    return None;

    //.collect_into_vec(&mut results);

    // if let Some((best_program, best_score)) = results
    //     .into_iter()
    //     .filter_map(|option| option)
    //     .min_by_key(|(_, score)| ordered_float::OrderedFloat(*score))
    // {
    //     eprintln!("Coord^2: {} -> {}", original_score, best_score);
    //     if best_score < original_score {
    //         return Some((best_program, best_score));
    //     }
    // }
    // return None;

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

pub fn try_removing_color_op_best(
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

pub fn try_removing_color_op_any(
    program: Program,
    initial_canvas: &Canvas,
    image: &Vec<Vec<Color>>,
) -> (Program, f64) {
    let original_score = score(&program, initial_canvas, image).unwrap();

    let ret = (0..program.len()).into_par_iter().find_map_any(|i| {
        if let Move::Color(_, _) = program[i] {
            let mut candidate_program = program.clone();
            candidate_program.remove(i);
            let candidate_program = optimize_color(candidate_program, initial_canvas, image);
            let candidate_score = score(&candidate_program, initial_canvas, image).unwrap();

            let candidate_program2 = remove_unnecessary_operations(&candidate_program);
            let candidate_score2 = score(&candidate_program2, initial_canvas, image).unwrap();
            assert!(candidate_score2 <= candidate_score);

            if candidate_score2 < original_score {
                return Some((candidate_program, candidate_score));
            }
        }
        None
    });

    ret.unwrap_or((program, original_score))
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

// internal functions for fix_cut_merge
fn ori_of_block(block: Block, ori: char) -> (i32, i32) {
    match ori {
        'x' | 'X' => (block.0 .0, block.1 .0),
        'y' | 'Y' => (block.0 .1, block.1 .1),
        _ => {
            unreachable!()
        }
    }
}

fn two_merge_cost(left: i32, right: i32, first: i32, second: i32) -> f64 {
    // 比較用なのでcanvas sizeを無視
    [
        if first < second {
            (first - left).max(second - first)
        } else {
            (first - second).max(right - first)
        },
        (second - left).max(right - second),
    ]
    .into_iter()
    .map(|v| 1.0 / v as f64)
    .sum()
}

pub fn fix_cut_merge_all(mut program: Program, canvas: &Canvas, image: &Vec<Vec<Color>>) -> (Program, f64) {
    let mut score = canvas.clone().apply_all_and_score(program.clone(), image).unwrap();
    while let Some(new_program) =
        fix_cut_merge(program.clone(), canvas.clone())
    {
        program = new_program;
        let new_score = canvas.clone().apply_all_and_score(program.clone(), image).unwrap();
        eprintln!("***** {score} -> {new_score}");
        assert!(new_score <= score);
        score = new_score;
    }
    return (program, score);
}

pub fn fix_cut_merge(mut program: Program, mut canvas: Canvas) -> Option<Program> {
    // let mut dirty_block_ids = HashSet::new();
    // let mut created_time = HashMap::new();
    // dirtyについては無効な値になっている
    let mut all_blocks = canvas.blocks.clone();
    let mut clean_blocks = HashMap::new();
    let mut all_ori = vec![None; program.len()];

    // for i in 0..num_start_blocks as u32 {
    //     created_time[BlockId(vec![i])] = !0;
    // }

    // for (bid, block) in canvas.blocks.iter() {
    //     all_blocks.insert(bid.clone(), block.clone());
    // }

    // for (t, mov) in program.iter().enumerate() {
    for t in 0..program.len() {
        let mov = program[t].clone();
        if let Move::Merge(bid0, bid1) = &mov {
            let corner0 = canvas.blocks[bid0].0;
            let corner1 = canvas.blocks[bid1].0;
            all_ori[t] = Some(if corner0.0 < corner1.0 {
                ('x', 0, corner1.0)
            } else if corner0.0 > corner1.0 {
                ('x', 1, corner0.0)
            } else if corner0.1 < corner1.1 {
                ('y', 0, corner1.1)
            } else if corner1.1 < corner0.1 {
                ('y', 1, corner0.1)
            } else {
                unreachable!()
            });
        }
        let info = (t, mov.clone());
        if !matches!(mov, Move::Color(..)) {
            // 高速化。色は関係ないので。
            // swapはidが変わるのでやる。
            canvas.apply(&mov);
        }
        match mov {
            Move::Color(bid, _) => {
                // 重複removeしてもOK
                clean_blocks.remove(&bid);
            }
            Move::Swap(bid0, bid1) => {
                // 重複removeしてもOK
                clean_blocks.remove(&bid0);
                clean_blocks.remove(&bid1);
            }
            Move::PointCut(bid, _, _) => {
                for childid in bid.cut4() {
                    all_blocks.insert(childid.clone(), canvas.blocks[&childid]);
                    clean_blocks.insert(childid.clone(), info.clone());
                }
            }
            Move::LineCut(bid, ori, val) => {
                let ori = ori.to_ascii_lowercase();
                for childid in bid.cut() {
                    all_blocks.insert(childid.clone(), canvas.blocks[&childid]);
                    clean_blocks.insert(childid.clone(), info.clone());
                }

                if let Some(&(s, Move::LineCut(ref parent_bid, parentori, parent_val))) =
                    clean_blocks.get(&bid)
                {
                    if ori == parentori.to_ascii_lowercase() {
                        let parent_block = all_blocks[&parent_bid];
                        let k = *bid.0.last().unwrap();
                        let other_bid = parent_bid.extended([1-k]);
                        if clean_blocks.contains_key(&other_bid) && canvas.blocks.contains_key(&other_bid) {
                            let (v0, v1) = ori_of_block(parent_block, ori);
                            // 端に近い方を先に切るべき
                            if (val - v0).min(v1 - val) < (parent_val - v0).min(v1 - parent_val) {
                                eprintln!("[{s}, {t}] cut-cut {ori}");
                                // let root_id = parent_bid.0.clone();

                                let tmp0 = parent_bid.extended([k]).0;
                                let tmp00 = parent_bid.extended([k, k]).0;
                                let tmp01 = parent_bid.extended([k, 1 - k]).0;
                                let tmp1 = parent_bid.extended([1 - k]).0;
                                let tmp10 = parent_bid.extended([1 - k, k]).0;
                                let tmp11 = parent_bid.extended([1 - k, 1 - k]).0;
                                // swap val and parent_val
                                program[s] = Move::LineCut(parent_bid.clone(), ori, val);
                                program[t] = Move::LineCut(BlockId(tmp1.clone()), ori, parent_val);

                                let from_to = [(tmp00, tmp0), (tmp01, tmp10), (tmp1, tmp11)];
                                let rename = |b: &mut Vec<u32>| {
                                    for (from, to) in from_to.iter() {
                                        if b.starts_with(from) {
                                            b.splice(..from.len(), to.clone());
                                            break;
                                        }
                                    }
                                };
                                for i in t + 1..program.len() {
                                    program[i].edit_id(rename);
                                }
                                return Some(program);
                            }
                        }
                    }
                }
            }
            Move::Merge(bid0, bid1) => {
                let (ori, _ori_index, val) = all_ori[t].unwrap();
                let newid_u32 = canvas.counter;
                let newid = BlockId(vec![newid_u32]);
                let new_block = canvas.blocks[&newid];
                all_blocks.insert(newid.clone(), new_block);
                clean_blocks.insert(newid.clone(), info);

                match (
                    clean_blocks.get(&bid0).cloned(),
                    clean_blocks.get(&bid1).cloned(),
                ) {
                    (Some((s0, Move::LineCut(..))), Some((s1, Move::LineCut(..)))) if s0 == s1 => {
                        let s = s0;
                        eprintln!("[{s}, {t}] cut-merge");
                        let mut old_id = bid0.0.clone();
                        old_id.pop();
                        let rename = |b: &mut Vec<u32>| {
                            if b[0] == newid_u32 {
                                b.splice(..1, old_id.clone());
                            } else if b[0] > newid_u32 {
                                b[0] -= 1;
                            }
                        };
                        for i in t + 1..program.len() {
                            program[i].edit_id(rename);
                        }
                        // s < t
                        program.remove(t);
                        program.remove(s);
                        return Some(program);
                    }
                    (Some((s, Move::Merge(id0, id1))), _)
                    | (_, Some((s, Move::Merge(id0, id1)))) => {
                        let (prev_ori, _, prev_val) = all_ori[s].unwrap();
                        if prev_ori == ori {
                            let (v0, v1) = ori_of_block(new_block, ori);
                            // let (v00, v01) = ori_of_block(all_blocks[&id0], ori);
                            // let (v10, v11) = ori_of_block(all_blocks[&id1], ori);
                            // let set0 = HashSet::<i32>::from_iter([v00, v01]);
                            // let set1 = HashSet::<i32>::from_iter([v10, v11]);
                            // let v_first = *set0.intersection(&set1).next().unwrap();
                            // let v_second = *set0.union(&set1).copied().collect::<HashSet<i32>>().difference(&HashSet::<i32>::from([v0, v1, v_first])).next().unwrap();
                            // assert_eq!(v_first, prev_val);
                            // assert_eq!(v_second, val);
                            if two_merge_cost(v0, v1, prev_val, val)
                                > two_merge_cost(v0, v1, val, prev_val)
                            {
                                eprintln!("[{s}, {t}] merge-merge {ori}");
                                // dbg!((v0, v1, prev_val, val));
                                eprintln!("auto-fix not yet implemented")
                            }
                        }
                    }
                    _ => {
                        // ok
                    }
                }
                // if let Some(&(s, Move::Merge(..))) = clean_blocks.get(&bid0) {
                //     if all_ori[s].unwrap().0 == ori {
                //         eprintln!("[{s}, {t}] merge-merge {ori}");
                //     }
                // }
                // if let Some(&(s, Move::Merge(..))) = clean_blocks.get(&bid1) {
                //     if all_ori[s].unwrap().0 == ori {
                //         eprintln!("[{s}, {t}] merge-merge {ori}");
                //     }
                // }
            }
        }
    }
    None
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
    max_pair_perturb: i32,
) -> (Program, f64) {
    let mut best_score = super::canvas::score(&best_program, initial_canvas, image).unwrap();
    if best_program.is_empty() {
        return (best_program, best_score);
    }
    // return fix_cut_merge_all(best_program, initial_canvas, image);

    let mut diff_step = 1;
    while diff_step <= max_diff_step {
        // (1) Try color improvement
        if diff_step == 1 {
            let (new_program, new_score) =
                try_removing_color_op_best(best_program.clone(), initial_canvas, &image);
            if new_score < best_score {
                eprintln!("Improvement - Color: {} -> {}", best_score, new_score);
                best_program = new_program;
                best_score = new_score;
                continue;
            }
        }

        // (2) Try coordinate improvement
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

        // (3) Try pair coordinate improvement
        if diff_step == max_diff_step + 1 {
            for d in 1..=max_pair_perturb {
                if let Some((new_program, new_score)) =
                    optimize_coord_two(&best_program, initial_canvas, image, d)
                {
                    if new_score < best_score {
                        eprintln!("Improvement - Pair coord: {} -> {}", best_score, new_score);
                        best_program = new_program;
                        best_score = new_score;
                        diff_step = 1;
                        break;
                    }
                }
            }
        }
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
