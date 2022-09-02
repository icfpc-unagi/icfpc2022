use super::{BlockId, Move};

fn block_id_push(mut block_id: BlockId, x: u32) -> BlockId {
    block_id.0.push(x);
    block_id
}

pub fn flip_png(mut png: Vec<Vec<[u8; 4]>>) -> Vec<Vec<[u8; 4]>> {
    // 左右反転
    for row in png.iter_mut() {
        row.reverse();
    }
    png
}

const width: i32 = 400;

pub fn flip_program(program: &Vec<Move>) -> Vec<Move> {
    let mut block_id_map = std::collections::HashMap::<BlockId, BlockId>::new();
    block_id_map.insert(BlockId(vec![0]), BlockId(vec![0]));

    let mut flipped_program = vec![];
    let mut n = 0;
    for mv in program {
        let flipped_mv;
        match mv {
            Move::LineCut(block_id, orientation, offset) => {
                if *orientation == 'y' || *orientation == 'Y' {
                    let block_id = block_id_map.get(block_id).unwrap().clone();
                    let block_id0 = block_id_push(block_id.clone(), 0);
                    let block_id1 = block_id_push(block_id.clone(), 1);
                    block_id_map.insert(block_id0.clone(), block_id0.clone());
                    block_id_map.insert(block_id1.clone(), block_id1.clone());

                    flipped_mv = Move::LineCut(block_id, *orientation, *offset);
                } else {
                    let block_id = block_id_map.get(block_id).unwrap().clone();
                    let block_id0 = block_id_push(block_id.clone(), 0);
                    let block_id1 = block_id_push(block_id.clone(), 1);
                    block_id_map.insert(block_id0.clone(), block_id1.clone());
                    block_id_map.insert(block_id1.clone(), block_id0.clone());

                    flipped_mv = Move::LineCut(block_id, *orientation, width - *offset);
                }
            }
            Move::PointCut(block_id, offset_x, offset_y) => {
                unimplemented!();
                let block_id = block_id_map.get(block_id).unwrap().clone();
                block_id_map.insert(
                    block_id_push(block_id.clone(), 0),
                    block_id_push(block_id.clone(), 1),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 1),
                    block_id_push(block_id.clone(), 0),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 2),
                    block_id_push(block_id.clone(), 3),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 3),
                    block_id_push(block_id.clone(), 2),
                );

                flipped_mv = Move::PointCut(block_id, width - *offset_x, *offset_y);
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
                n += 1;
                block_id_map.insert(BlockId(vec![n]), BlockId(vec![n]));

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

    let mut flipped_program = vec![];
    let mut n = 0;
    for mv in program {
        let flipped_mv;
        println!("\n\n\n");
        dbg!(&block_id_map);
        dbg!(&mv);

        match mv {
            Move::LineCut(block_id, orientation, offset) => {
                if *orientation == 'y' || *orientation == 'Y' {
                    let block_id = block_id_map.get(block_id).unwrap().clone();
                    let block_id0 = block_id_push(block_id.clone(), 0);
                    let block_id1 = block_id_push(block_id.clone(), 1);
                    block_id_map.insert(block_id0.clone(), block_id0.clone());
                    block_id_map.insert(block_id1.clone(), block_id1.clone());

                    flipped_mv = Move::LineCut(block_id, 'x', *offset);
                } else {
                    let block_id = block_id_map.get(block_id).unwrap().clone();
                    let block_id0 = block_id_push(block_id.clone(), 0);
                    let block_id1 = block_id_push(block_id.clone(), 1);
                    block_id_map.insert(block_id0.clone(), block_id1.clone());
                    block_id_map.insert(block_id1.clone(), block_id0.clone());

                    flipped_mv = Move::LineCut(block_id, 'y', width - *offset);
                }
            }
            Move::PointCut(block_id, offset_x, offset_y) => {
                unimplemented!();
                let block_id = block_id_map.get(block_id).unwrap().clone();

                // TODO: 結構自信ないよｗｗ
                block_id_map.insert(
                    block_id_push(block_id.clone(), 0),
                    block_id_push(block_id.clone(), 1),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 1),
                    block_id_push(block_id.clone(), 2),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 2),
                    block_id_push(block_id.clone(), 3),
                );
                block_id_map.insert(
                    block_id_push(block_id.clone(), 3),
                    block_id_push(block_id.clone(), 0),
                );

                flipped_mv = Move::PointCut(block_id, *offset_y, width - *offset_x);
            }
            Move::Color(block_id, color) => {
                dbg!(&block_id);
                flipped_mv = Move::Color(block_id_map.get(block_id).unwrap().clone(), color.clone())
            }
            Move::Swap(block_id1, block_id2) => {
                flipped_mv = Move::Swap(
                    block_id_map.get(block_id1).unwrap().clone(),
                    block_id_map.get(block_id2).unwrap().clone(),
                )
            }
            Move::Merge(block_id1, block_id2) => {
                n += 1;
                block_id_map.insert(BlockId(vec![n]), BlockId(vec![n]));
                // dbg!(&block_id1, block_id2);

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
