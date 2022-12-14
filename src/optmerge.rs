use std::mem;

use itertools::Itertools;

use crate::{mat, Block, BlockId, Canvas, CostType, Move, Point, SetMinMax};

pub fn merge_all(canvas: &mut Canvas) -> Vec<Move> {
    merge_all_safe(canvas).unwrap()
}

fn merge_all_safe(canvas: &mut Canvas) -> anyhow::Result<Vec<Move>> {
    if canvas.blocks.len() == 1 {
        // do nothing
        return Ok(vec![]);
    };
    assert_eq!(canvas.cost_type, CostType::Basic);

    let canvas_h = canvas.bitmap.len();
    let canvas_w = canvas.bitmap[0].len();

    let Block(Point(x0, y0), Point(x1, y1)) = canvas.blocks.values().next().unwrap();
    let block_h = y1 - y0;
    let block_w = x1 - x0;

    let h = canvas_h / block_h as usize;
    let w = canvas_w / block_w as usize;

    let mut block_ids = mat![None; h; w];
    for (id, block) in canvas.blocks.iter() {
        let Block(Point(x0, y0), _) = block;
        let i = y0 / block_h;
        let j = x0 / block_w;
        if block
            != &Block(
                Point(block_w * j, block_h * i),
                Point(block_w * (j + 1), block_h * (i + 1)),
            )
        {
            anyhow::bail!("unexpected position of block");
        }
        block_ids[i as usize][j as usize] = Some(id.clone());
    }

    // if h != w {
    //     bail!("not implemented yet...");
    // }

    let (cost, path) = merge_all_internal(h, w);
    eprintln!("{:?}", path);
    dbg!(cost);

    let mut id_a = BlockId(vec![]); // invalid id
    let mut id_b = BlockId(vec![]); // invalid id
    let mut moves = vec![];

    let mut canvas_counter = canvas.counter;
    let mut new_id = || {
        canvas_counter += 1;
        BlockId(vec![canvas_counter])
    };
    let mut actual_cost = 0.0;
    let mut push = |mov: Move| {
        // eprintln!("{:?}", &mov);
        actual_cost += canvas.apply(&mov);
        moves.push(mov);
    };
    let cut_a = |id, a: usize| Move::LineCut(id, 'y', block_h * a as i32);
    let cut_b = |id, b: usize| Move::LineCut(id, 'x', block_w * b as i32);

    let mut r_prev = -1;
    for (a, b, r) in path {
        // eprintln!("{:?}", (a, b, r));
        match r {
            0 => {
                let mut id_tmp = if b == 0 {
                    block_ids[a][0].take().unwrap()
                } else if a == h - 1 {
                    id_b.clone()
                } else if r_prev == r || a == 0 {
                    let [id0, id1] = id_b.cut();
                    push(cut_a(id_b.clone(), a + 1));
                    id_b = id1;
                    id0
                } else {
                    let (id0, id1, id2) = if a + 1 < h - a {
                        let [id0, id12] = id_b.cut();
                        push(cut_a(id_b.clone(), a));
                        let [id1, id2] = id12.cut();
                        push(cut_a(id12, a + 1));
                        (id0, id1, id2)
                    } else {
                        let [id01, id2] = id_b.cut();
                        push(cut_a(id_b.clone(), a + 1));
                        let [id0, id1] = id01.cut();
                        push(cut_a(id01, a));
                        (id0, id1, id2)
                    };
                    push(Move::Merge(id0, mem::replace(&mut id_a, new_id())));
                    id_b = id2;
                    id1
                };
                for bb in b.max(1)..w {
                    push(Move::Merge(
                        mem::replace(&mut id_tmp, new_id()),
                        block_ids[a][bb].take().unwrap(),
                    ));
                }
                if a > 0 {
                    push(Move::Merge(mem::replace(&mut id_a, new_id()), id_tmp));
                } else {
                    id_a = id_tmp;
                }
            }
            1 => {
                let mut id_tmp = if a == 0 {
                    block_ids[0][b].take().unwrap()
                } else if b == w - 1 {
                    id_a.clone()
                } else if r_prev == r || b == 0 {
                    let [id0, id1] = id_a.cut();
                    push(cut_b(id_a.clone(), b + 1));
                    id_a = id1;
                    id0
                } else {
                    let (id0, id1, id2) = if b + 1 < w - b {
                        let [id0, id12] = id_a.cut();
                        push(cut_b(id_a.clone(), b));
                        let [id1, id2] = id12.cut();
                        push(cut_b(id12, b + 1));
                        (id0, id1, id2)
                    } else {
                        let [id01, id2] = id_a.cut();
                        push(cut_b(id_a.clone(), b + 1));
                        let [id0, id1] = id01.cut();
                        push(cut_b(id01, b));
                        (id0, id1, id2)
                    };
                    push(Move::Merge(id0, mem::replace(&mut id_b, new_id())));
                    id_a = id2;
                    id1
                };
                for aa in a.max(1)..h {
                    push(Move::Merge(
                        mem::replace(&mut id_tmp, new_id()),
                        block_ids[aa][b].take().unwrap(),
                    ));
                }
                if b > 0 {
                    push(Move::Merge(mem::replace(&mut id_b, new_id()), id_tmp));
                } else {
                    id_b = id_tmp;
                }
            }
            _ => {
                break;
            }
        }
        r_prev = r;
    }
    assert_eq!(canvas.blocks.len(), 1);
    assert_eq!(actual_cost as i32, cost);
    Ok(moves)
}

fn merge_all_internal(h: usize, w: usize) -> (i32, Vec<(usize, usize, i8)>) {
    const INF: i32 = 1_000_000_000;
    // (min_cost, reproduce_key)
    let mut cost0 = mat![(INF, -1); h+1; w+1];
    let mut cost1 = mat![(INF, -1); h+1; w+1];

    let cost_scale = (h * w) as f64;
    // CostType::Basic is assumed here!!
    let cut_cost = |a: usize, b: usize| (7.0 * cost_scale / ((a * b) as f64)).round() as i32;
    let merge_cost = |a: usize, b: usize| (1.0 * cost_scale / ((a * b) as f64)).round() as i32;

    for (a, b) in itertools::iproduct!(0..=h, 0..=w)
        .sorted_by_key(|&(a, b)| (a + b, a))
        .rev()
    {
        // dbg!(a, b);
        if a == h {
            cost0[a][b].0 = 0;
        }
        if b == w {
            cost1[a][b].0 = 0;
        }
        if a == h || b == w {
            continue;
        }
        if a < h {
            let rep = 0;
            let common_cost = {
                let mut c: i32 = (b.max(1)..w).map(|bb| merge_cost(1, bb)).sum();
                if a > 0 {
                    c += merge_cost(a, w);
                }
                c += cost0[a + 1][b].0;
                c
            };
            if b == 0 || a == h - 1 {
                cost0[a][b].setmin((common_cost, rep));
            } else {
                cost0[a][b].setmin((common_cost + cut_cost(h - a, b), rep));
                let mut c = cut_cost(h, b);
                if a > 0 {
                    c += cut_cost((a + 1).max(h - a), b) + merge_cost(a, b.max(w - b));
                }
                cost1[a][b].setmin((common_cost + c, rep));
            }
        }
        if b < w {
            let rep = 1;
            let common_cost = {
                let mut c: i32 = (a.max(1)..h).map(|aa| merge_cost(aa, 1)).sum();
                if b > 0 {
                    c += merge_cost(h, b);
                }
                c += cost1[a][b + 1].0;
                c
            };
            if a == 0 || b == w - 1 {
                cost1[a][b].setmin((common_cost, rep));
            } else {
                cost1[a][b].setmin((common_cost + cut_cost(a, w - b), rep));
                let mut c = cut_cost(a, w);
                if b > 0 {
                    c += cut_cost(a, (b + 1).max(w - b)) + merge_cost(a.max(h - a), b);
                }
                cost0[a][b].setmin((common_cost + c, rep));
            }
        }
        // dbg!(cost0[a][b]);
        // dbg!(cost1[a][b]);
    }
    let mut a = 0;
    let mut b = 0;
    let (total_cost, mut r) = std::cmp::min(cost0[a][b], cost1[a][b]);
    let mut ret = vec![];
    loop {
        ret.push((a, b, r));
        match r {
            0 => {
                a += 1;
                r = cost0[a][b].1;
            }
            1 => {
                b += 1;
                r = cost1[a][b].1;
            }
            _ => {
                assert!(a == h || b == w);
                return (total_cost, ret);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_problem;

    #[test]
    fn test_merge_all() {
        let (mut canvas, _) = load_problem(26);
        merge_all(&mut canvas);
    }

    #[test]
    fn test_internal_cost_only() {
        // ???????????????????????????????????????????????????????????????????????????
        assert_eq!(merge_all_internal(10, 10).0, 2426);
        assert_eq!(merge_all_internal(20, 20).0, 17571);
        // ????????????????????????????????????????????????
        assert_eq!(merge_all_internal(10, 30).0, 10176);
        assert_eq!(merge_all_internal(40, 10).0, 14617);
    }
}
