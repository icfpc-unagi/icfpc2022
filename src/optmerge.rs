use itertools::Itertools;

use crate::{Canvas, Move, Block, Point, mat, BlockId, SetMinMax};

pub fn merge_all(canvas: &mut Canvas) -> Vec<Move> {
    merge_all_safe(canvas).unwrap()
}

fn merge_all_safe(canvas: &mut Canvas) -> anyhow::Result<Vec<Move>> {
    let canvas_h = canvas.bitmap.len();
    let canvas_w = canvas.bitmap[0].len();

    let Block(Point(x0, y0), Point(x1, y1))=  canvas.blocks.values().next().unwrap();
    let block_h = (y1- y0);
    let block_w = (x1- x0);

    let h = canvas_h / block_h as usize;
    let w = canvas_w / block_w as usize;

    let mut block_ids = mat![BlockId(vec![]); h; w];
    for (id, block) in canvas.blocks.iter() {
        let Block(Point(x0, y0), Point(x1, y1)) = block;
        let i = y0 / block_h;
        let j = x0 / block_w;
        if block != &Block(Point(block_w * j, block_h * i), Point(block_w * (j+1), block_h * (i+1))) {
            anyhow::bail!("unexpected position of block");
        }
        block_ids[i as usize][j as usize] = id.clone();
    }
    let block_ids = block_ids;

    // if h != w {
    //     bail!("not implemented yet...");
    // }

    merge_all_internal(h, w);

    let mut moves = vec![];
    Ok((moves))
}

fn merge_all_internal(h: usize, w: usize) {
    const INF: i32 = 1_000_000_000;
    let mut cost0 = mat![INF; h+1; w+1];
    let mut cost1 = mat![INF; h+1; w+1];

    let cost_scale = (h * w) as f64;
    let cut_cost = |a: usize, b: usize| (7.0 * cost_scale / ((a * b) as f64)).round() as i32;
    let merge_cost = |a: usize, b: usize| (1.0 * cost_scale / ((a * b) as f64)).round() as i32;

    for (a, b) in itertools::iproduct!(0..=h, 0..=w).sorted_by_key(|&(a, b)| (a+b, a)).rev() {
        // dbg!(a, b);
        if (a, b) == (h, w) {
            cost0[a][b] = 0;
            cost1[a][b] = 0;
            continue;
        }
        if a < h {
            let mut c = cut_cost(h-a, b) + (b..w).map(|bb| merge_cost(1, bb)).sum::<i32>();
            if a > 0 {
                c += merge_cost(a, w);
            }
            c += cost0[a+1][b];
            cost0[a][b].setmin(c);
        }
    }
}