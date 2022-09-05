use std::collections::HashMap;

//use crate::optmerge::merge_all;
//use crate::wata::all_merge;
use crate::{color::*, wata::merge_solution};
//use crate::wata::*;
use crate::*;
use std::time::{Duration, Instant};

//use std::collections::HashMap;

pub fn monte_solve(png: &Vec<Vec<[u8; 4]>>, sec: i32) -> (f64, Program) {
    return monte_solve2(png, sec, &Canvas::new(png[0].len(), png.len()));
}

pub fn monte_solve2(png: &Vec<Vec<[u8; 4]>>, sec: i32, init_canvas: &Canvas) -> (f64, Program) {
    let mut map: HashMap<i64, usize> = HashMap::new();
    let mut list = vec![];
    let mut best = 999999999.0;
    let mcs = MedianColorSelector::new(png);
    let mut xdp = vec![vec![0.0; 401]; 400];
    let mut ydp = vec![vec![0.0; 401]; 400];
    let mut cut_cost = 7.0;
    if init_canvas.cost_type == CostType::V2 {
        cut_cost = 2.0;
    }

    for i in 1..400 {
        for j in 0..400 {
            ydp[i][j + 1] = color_dist(png[i][j], png[i - 1][j]);
            xdp[i][j + 1] = color_dist(png[j][i], png[j][i - 1]);
        }

        for j in 0..400 {
            ydp[i][j + 1] += ydp[i][j];
            xdp[i][j + 1] += xdp[i][j];
        }
    }

    eprintln!("sec: {}", sec);
    let start = Instant::now();
    for i in 0..200000000 {
        let end = start.elapsed();
        if end >= Duration::from_secs(sec as u64) {
            break;
        }
        let ret = search(
            0, 400, 0, 400, &mut map, &mut list, &png, &mcs, &ydp, &xdp, cut_cost,
        );
        let ret_score = ret.0;
        if best > ret_score {
            best = ret_score;
            //eprintln!(
            //    "time:{}   score:{}    node:{}",
            //    end.as_millis(),
            //    best,
            //    list.len()
            //);
        }
        //eprintln!("cnt:{}   score:{}", cnt, best);
    }
    eprintln!("score:{}    node:{}", best, list.len());

    let mut canvas2 = init_canvas.clone();
    let mut start_id = 0;
    //let mv = all_merge(&mut canvas2).1;
    //let mv = merge_all(&mut canvas2);
    // let mv = merge_all(&mut canvas2);
    let mut moves = vec![];

    /*
    for m in &moves {
        match m {
            Move::Merge(_, _) => start_id += 1,
            _ => {}
        }
        //eprintln!("{}", m);
    }
    */

    //eprintln!("start : {}", start_id);

    let mut blocks = vec![BlockId(vec![0])];

    eprintln!("check");

    let _ = search2(
        0,
        400,
        0,
        400,
        &mut map,
        &mut list,
        &png,
        &mut moves,
        &mut blocks,
        start_id as usize,
    );

    eprintln!("ok");

    return (0.0, moves);
}

fn color_dist(a: [u8; 4], b: [u8; 4]) -> f64 {
    let mut sum = 0.0;
    for i in 0..4 {
        let c = (a[i] as f64) - (b[i] as f64);
        sum = sum + c * c;
    }
    return sum.sqrt();
}

fn search2(
    ly: usize,
    ry: usize,
    lx: usize,
    rx: usize,
    map: &mut HashMap<i64, usize>,
    list: &mut Vec<Node>,
    png: &Vec<Vec<[u8; 4]>>,
    moves: &mut Vec<Move>,
    blocks: &mut Vec<BlockId>,
    id: usize,
) -> usize {
    //eprintln!("{} {} {} {}", ly, ry, lx, rx);

    let hash = (((ly * 65536 + ry) as i64) << 31) + (lx * 65536 + rx) as i64;
    let now = map[&hash];
    let mut id2 = id;

    if list[now].def == list[now].best {
        let block = blocks.pop().unwrap();
        moves.push(Move::Color(
            block.clone(),
            best_color2(&png, lx, rx, ly, ry).0,
        ));
        blocks.push(block);
    } else {
        //check

        //left
        let left = list[now].next[0].1;
        id2 = search2(
            left.1[0],
            left.1[1],
            left.1[2],
            left.1[3],
            map,
            list,
            png,
            moves,
            blocks,
            id2.clone(),
        );
        //Line
        let block = blocks.pop().unwrap();
        moves.push(Move::LineCut(
            block.clone(),
            list[now].next[0].0 .1,
            list[now].next[0].0 .2 as i32,
        ));
        let bar = block.cut();
        blocks.push(bar[0].clone());
        blocks.push(bar[1].clone());

        //Right
        let right = list[now].next[0].2;
        id2 = search2(
            right.1[0],
            right.1[1],
            right.1[2],
            right.1[3],
            map,
            list,
            png,
            moves,
            blocks,
            id2.clone(),
        );
        //Merge
        let b1 = blocks.pop().unwrap();
        let b2 = blocks.pop().unwrap();
        moves.push(Move::Merge(b1, b2));
        //あとでなおす
        id2 += 1;
        blocks.push(BlockId(vec![(id2) as u32]));
    }
    id2
}

fn search(
    ly: usize,
    ry: usize,
    lx: usize,
    rx: usize,
    map: &mut HashMap<i64, usize>,
    list: &mut Vec<Node>,
    png: &Vec<Vec<[u8; 4]>>,
    mcs: &MedianColorSelector,
    ydp: &Vec<Vec<f64>>,
    xdp: &Vec<Vec<f64>>,
    cut_cost: f64,
) -> (f64, bool) {
    let hash = (((ly * 65536 + ry) as i64) << 31) + (lx * 65536 + rx) as i64;
    if !map.contains_key(&hash) {
        map.insert(hash, list.len());
        list.push(Node {
            cnt: 0,
            best: 0.0,
            //def: median_color(png, lx, rx, ly, ry).1 * 0.005
            //    + 5.0 * 400.0 * 400.0 / (400.00001 - ly as f64) / (400.00001 - lx as f64),
            def: mcs.query(lx, rx, ly, ry).1 as f64 * 0.005 * 0.66
                + 5.0 * 400.0 * 400.0 / (400.00001 - ly as f64) / (400.00001 - lx as f64),
            target: 0,
            used: false,
            ok_count: 0,
            next: vec![],
        });
    }

    if lx == 400 || ly == 400 || lx >= rx || ly >= ry {
        eprintln!("?????? {} {} {} {}", ly, lx, ry, rx);

        return (99999999999.9, true);
    }

    //eprintln!("c1 {} {} {} {}", ly, ry, lx, rx);

    let now = map[&hash];

    if list[now].used {
        return (list[now].best, list[now].used);
    }

    if list[now].cnt == 0 {
        list[now].best = list[now].def;

        let dy = ry - ly;
        let dx = rx - lx;
        //let bc = best_color(&png, lx, rx, ly, ry);

        let mut div_pos = vec![]; // (score, XorY, pos)

        let p = 0;
        let minimum_cost = (2.0 * cut_cost + 8.0) * 400.0 * 400.0
            / (400.00001 - ly as f64)
            / (400.00001 - lx as f64);

        if minimum_cost < list[now].def {
            if dx >= (1 << p) * 2 {
                for i in lx + (1 << p)..rx - (1 << p) + 1 {
                    let mut score = xdp[i][ry] - xdp[i][ly];
                    score *= dx as f64;

                    div_pos.push((score, 'X', i));
                }
            }
            if dy >= (1 << p) * 2 {
                for i in ly + (1 << p)..ry - (1 << p) + 1 {
                    let mut score = ydp[i][rx] - ydp[i][lx];
                    score *= dy as f64;

                    div_pos.push((score, 'Y', i));
                }
            }
        }

        div_pos.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        list[now].target = 32;

        if list[now].target > div_pos.len() {
            list[now].target = div_pos.len();
        }

        for i in 0..list[now].target {
            let mut div = div_pos[i];
            let left;
            let right;
            if div.1 == 'X' {
                left = (false, [ly, ry, lx, div.2]);
                right = (false, [ly, ry, div.2, rx]);
            } else {
                left = (false, [ly, div.2, lx, rx]);
                right = (false, [div.2, ry, lx, rx]);
            }
            div.0 = 9999999999.0;
            list[now].next.push((div, left, right));
        }
    }

    if list[now].next.len() == 0 {
        list[now].used = true;
        return (list[now].best, list[now].used);
    }

    let mut choice;
    list[now].cnt += 1;
    if list[now].target > list[now].cnt - 1 {
        choice = list[now].cnt - 1;
        //list[now].target += 1;
    } else {
        if list[now].cnt <= 10000000 {
            choice = bit_count(list[now].cnt) % list[now].target;
        } else {
            choice = list[now].cnt % list[now].target;
        }
    }
    for _ in 0..list[now].target {
        if list[now].next[choice].1 .0 && list[now].next[choice].2 .0 {
            choice += 1;
            choice %= list[now].target;
        }
    }

    let left = list[now].next[choice].1;
    let right = list[now].next[choice].2;

    let leftret = search(
        left.1[0], left.1[1], left.1[2], left.1[3], map, list, png, mcs, ydp, xdp, cut_cost,
    );
    let rightret = search(
        right.1[0], right.1[1], right.1[2], right.1[3], map, list, png, mcs, ydp, xdp, cut_cost,
    );
    let leftret_score = leftret.0;
    let rightret_score = rightret.0;

    list[now].next[choice].1 .0 |= leftret.1;
    list[now].next[choice].2 .0 |= rightret.1;

    if leftret.1 && rightret.1 {
        list[now].ok_count += 1;
        if list[now].ok_count == list[now].target {
            list[now].used = true;
        }
    }

    let s = {
        let s1 = (400.001 - right.1[0] as f64) * (400.001 - right.1[2] as f64);
        let s2 = (400.001 - left.1[0] as f64) * (400.001 - left.1[2] as f64) - s1;
        if s1 > s2 {
            s1
        } else {
            s2
        }
    };

    let ret = leftret_score
        + rightret_score
        + (400.0 * 400.0 / s) * 1.0
        + (400.0 * 400.0 / ((400.00001 - lx as f64) * (400.00001 - ly as f64))) * cut_cost;

    if list[now].next[choice].0 .0 > ret {
        list[now].next[choice].0 .0 = ret;
        list[now]
            .next
            .sort_by(|a, b| (a.0 .0).partial_cmp(&b.0 .0).unwrap());

        if list[now].best > list[now].next[0].0 .0 {
            list[now].best = list[now].next[0].0 .0;
        }
    }

    if false && ly == 0 && lx == 0 && ry == 400 && rx == 400 {
        eprintln!(
            "cnt:{} ({},{})-({},{}):{} ({}) for (({},{})({},{})), (({},{}),({},{})) {} {} {}",
            list[now].cnt,
            ly,
            lx,
            ry,
            rx,
            list[now].best,
            ret,
            left.1[0],
            left.1[2],
            left.1[1],
            left.1[3],
            right.1[0],
            right.1[2],
            right.1[1],
            right.1[3],
            leftret_score,
            rightret_score,
            list[now].def
        );
    }
    return (list[now].best, list[now].used);
}

fn bit_count(a: usize) -> usize {
    if a % 2 == 0 {
        return 0;
    } else {
        return bit_count(a / 2) + 1;
    }
}

#[derive(Clone, Debug)]
struct Node {
    cnt: usize,
    best: f64,
    def: f64,
    target: usize,
    used: bool,
    ok_count: usize,
    next: Vec<((f64, char, usize), (bool, [usize; 4]), (bool, [usize; 4]))>,
}
