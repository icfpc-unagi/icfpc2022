use super::{BlockId, Canvas, Move};
use std::collections::HashMap;

const WIDTH: i32 = 400;

fn push(block_id: &BlockId, x: u32) -> BlockId {
    block_id.extended([x])
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Flip
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn flip_png(mut png: Vec<Vec<[u8; 4]>>) -> Vec<Vec<[u8; 4]>> {
    // 左右反転
    for row in png.iter_mut() {
        row.reverse();
    }
    png
}

pub fn flip_program(program: &Vec<Move>) -> Vec<Move> {
    let mut block_id_map = std::collections::HashMap::<BlockId, BlockId>::new();
    block_id_map.insert(BlockId(vec![0]), BlockId(vec![0]));
    flip_program_with_initial_block_id_map(program, block_id_map, 0)
}

pub fn flip_program_with_initial_canvas(program: &Vec<Move>, canvas: &Canvas) -> Vec<Move> {
    let block_id_map = canvas
        .blocks
        .keys()
        .map(|block_id| (block_id.clone(), block_id.clone()))
        .collect();
    flip_program_with_initial_block_id_map(program, block_id_map, canvas.counter)
}

fn flip_program_with_initial_block_id_map(
    program: &Vec<Move>,
    mut block_id_map: HashMap<BlockId, BlockId>,
    mut global_counter: u32,
) -> Vec<Move> {
    // global_counterは「今ある最大のやつ」（次できるやつではない）、Canvas.counterと仕様一致してるはず

    let mut flipped_program = vec![];
    for mv in program {
        let flipped_mv;
        match mv {
            Move::LineCut(bid_old, orientation, offset) => {
                if *orientation == 'y' || *orientation == 'Y' {
                    let bid_new = block_id_map.get(bid_old).unwrap().clone();
                    block_id_map.insert(push(bid_old, 0), push(&bid_new, 0));
                    block_id_map.insert(push(bid_old, 1), push(&bid_new, 1));
                    flipped_mv = Move::LineCut(bid_new, *orientation, *offset);
                } else {
                    let bid_new = block_id_map.get(bid_old).unwrap().clone();
                    block_id_map.insert(push(bid_old, 0), push(&bid_new, 1));
                    block_id_map.insert(push(bid_old, 1), push(&bid_new, 0));
                    flipped_mv = Move::LineCut(bid_new, *orientation, WIDTH - *offset);
                }
            }
            Move::PointCut(_, _, _) => {
                unimplemented!();
                // let block_id = block_id_map.get(block_id).unwrap().clone();
                // block_id_map.insert(push(block_id.clone(), 0), push(block_id.clone(), 1));
                // block_id_map.insert(push(block_id.clone(), 1), push(block_id.clone(), 0));
                // block_id_map.insert(push(block_id.clone(), 2), push(block_id.clone(), 3));
                // block_id_map.insert(push(block_id.clone(), 3), push(block_id.clone(), 2));
                //
                // flipped_mv = Move::PointCut(block_id, width - *offset_x, *offset_y);
            }
            Move::Color(block_id, color) => {
                flipped_mv = Move::Color(block_id_map.get(block_id).unwrap().clone(), color.clone())
            }
            Move::Swap(block_id1, block_id2) => {
                flipped_mv = Move::Swap(
                    block_id_map.get(block_id1).unwrap().clone(),
                    block_id_map.get(block_id2).unwrap().clone(),
                )
            }
            Move::Merge(block_id1, block_id2) => {
                global_counter += 1;
                block_id_map.insert(BlockId(vec![global_counter]), BlockId(vec![global_counter]));

                flipped_mv = Move::Merge(
                    block_id_map.get(block_id1).unwrap().clone(),
                    block_id_map.get(block_id2).unwrap().clone(),
                );
            }
        }

        flipped_program.push(flipped_mv);
    }

    return flipped_program;
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Rotate
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn rotate_png(png: &Vec<Vec<[u8; 4]>>) -> Vec<Vec<[u8; 4]>> {
    let mut out = vec![vec![[0_u8; 4]; png.len()]; png[0].len()];
    for y in 0..png.len() {
        for x in 0..png[y].len() {
            out[png[0].len() - x - 1][y] = png[y][x];
        }
    }
    out
}

pub fn rotate_program(program: &Vec<Move>) -> Vec<Move> {
    let mut block_id_map = std::collections::HashMap::<BlockId, BlockId>::new();
    block_id_map.insert(BlockId(vec![0]), BlockId(vec![0]));
    rotate_program_with_initial_block_id_map(program, block_id_map, 0)
}

pub fn rotate_program_with_initial_canvas(program: &Vec<Move>, canvas: &Canvas) -> Vec<Move> {
    let block_id_map = canvas
        .blocks
        .keys()
        .map(|block_id| (block_id.clone(), block_id.clone()))
        .collect();
    rotate_program_with_initial_block_id_map(program, block_id_map, canvas.counter)
}

fn rotate_program_with_initial_block_id_map(
    program: &Vec<Move>,
    mut block_id_map: HashMap<BlockId, BlockId>,
    mut global_counter: u32,
) -> Vec<Move> {
    let mut flipped_program = vec![];
    for mv in program {
        let flipped_mv;
        match mv {
            Move::LineCut(bid_old, orientation, offset) => {
                if *orientation == 'y' || *orientation == 'Y' {
                    let bid_new = block_id_map.get(bid_old).unwrap().clone();
                    block_id_map.insert(push(bid_old, 0), push(&bid_new, 0));
                    block_id_map.insert(push(bid_old, 1), push(&bid_new, 1));
                    flipped_mv = Move::LineCut(bid_new, 'x', *offset);
                } else {
                    let bid_new = block_id_map.get(bid_old).unwrap().clone();
                    block_id_map.insert(push(bid_old, 0), push(&bid_new, 1));
                    block_id_map.insert(push(bid_old, 1), push(&bid_new, 0));
                    flipped_mv = Move::LineCut(bid_new, 'y', WIDTH - *offset);
                }
            }
            Move::PointCut(_, _, _) => {
                unimplemented!();
                // let block_id = block_id_map.get(block_id).unwrap().clone();
                //
                // // TODO: 結構自信ないよｗｗ
                // block_id_map.insert(push(block_id.clone(), 0), push(block_id.clone(), 1));
                // block_id_map.insert(push(block_id.clone(), 1), push(block_id.clone(), 2));
                // block_id_map.insert(push(block_id.clone(), 2), push(block_id.clone(), 3));
                // block_id_map.insert(push(block_id.clone(), 3), push(block_id.clone(), 0));
                //
                // flipped_mv = Move::PointCut(block_id, *offset_y, width - *offset_x);
            }
            Move::Color(block_id, color) => {
                // dbg!(&block_id);
                flipped_mv = Move::Color(block_id_map.get(block_id).unwrap().clone(), color.clone())
            }
            Move::Swap(block_id1, block_id2) => {
                flipped_mv = Move::Swap(
                    block_id_map.get(block_id1).unwrap().clone(),
                    block_id_map.get(block_id2).unwrap().clone(),
                )
            }
            Move::Merge(block_id1, block_id2) => {
                global_counter += 1;
                block_id_map.insert(BlockId(vec![global_counter]), BlockId(vec![global_counter]));
                // dbg!(&block_id1, block_id2);

                flipped_mv = Move::Merge(
                    block_id_map.get(block_id1).unwrap().clone(),
                    block_id_map.get(block_id2).unwrap().clone(),
                );
            }
        }

        flipped_program.push(flipped_mv);
    }

    flipped_program
}
