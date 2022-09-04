use std::collections::HashMap;

use crate::color::*;
use crate::wata::*;
use crate::*;
use std::time::{Duration, Instant};

//use std::collections::HashMap;

pub fn monte_solve(png: &mut Vec<Vec<[u8; 4]>>, sec: i32) -> (f64, Program) {
    return monte_solve2(png, sec, &Canvas::new(png[0].len(), png.len()));
}

pub fn monte_solve2(png: &mut Vec<Vec<[u8; 4]>>, sec: i32, init_canvas: &Canvas) -> (f64, Program) {
    let mut map: HashMap<i64, usize> = HashMap::new();
    let mut list = vec![];
    let mut best = 999999999.0;
    let start = Instant::now();
    for i in 0..200000000 {
        let end = start.elapsed();
        if end >= Duration::from_secs(sec as u64) {
            break;
        }
        let ret = search(0, 400, 0, 400, &mut map, &mut list, &png);
        if best > ret {
            best = ret;
            eprintln!("cnt:{}   score:{}    node:{}", i, best, list.len());
        }
        //eprintln!("cnt:{}   score:{}", cnt, best);
    }
    eprintln!("score:{}    node:{}", best, list.len());

    let (start_id, mv) = all_merge(&init_canvas);
    let mut moves = mv.clone();
    let mut blocks = vec![BlockId(vec![start_id])];

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

    return (0.0, moves);
    //for s in moves {
    //    println!("{}", s);
    //}
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
) -> f64 {
    let hash = (((ly * 65536 + ry) as i64) << 31) + (lx * 65536 + rx) as i64;
    if !map.contains_key(&hash) {
        map.insert(hash, list.len());
        list.push(Node {
            cnt: 0,
            best: 0.0,
            def: median_color(png, lx, rx, ly, ry).1 * 0.005
                + 5.0 * 400.0 * 400.0 / (400.00001 - ly as f64) / (400.00001 - lx as f64),
            target: 0,
            next: vec![],
        });
    }

    if lx == 400 || ly == 400 || lx >= rx || ly >= ry {
        eprintln!("?????? {} {} {} {}", ly, lx, ry, rx);
        return 99999999999.9;
    }

    //eprintln!("c1 {} {} {} {}", ly, ry, lx, rx);

    let now = map[&hash];

    if list[now].cnt == 0 {
        list[now].best = list[now].def;

        let dy = ry - ly;
        let dx = rx - lx;
        //let bc = best_color(&png, lx, rx, ly, ry);

        let mut div_pos = vec![]; // (score, XorY, pos)

        let p = 0;
        let minimum_cost = 18.0 * 400.0 * 400.0 / (400.00001 - ly as f64) / (400.00001 - lx as f64);

        if minimum_cost < list[now].def {
            if dx >= (1 << p) * 2 {
                for i in lx + (1 << p)..rx - (1 << p) + 1 {
                    let mut score = 0.00001;
                    for j in ly..ry {
                        let mut over = [0.0, 0.0, 0.0, 0.0];
                        let mut under = [0.0, 0.0, 0.0, 0.0];
                        let mut q = 1;
                        for k in 0..(1 << p) {
                            for c in 0..4 {
                                over[c] += png[j][i + k][c] as f64;
                                under[c] += png[j][i - 1 - k][c] as f64;
                            }
                            if k == q - 1 {
                                let mut tmp = 0.0;
                                for c in 0..4 {
                                    tmp += (over[c] - under[c]) * (over[c] - under[c]);
                                }
                                if k == 0 {
                                    tmp *= 25.0;
                                }
                                q *= 2;
                                score += tmp.sqrt();
                            }
                        }
                    }
                    //if rx - i < i - lx {
                    //    score *= (rx - i) as f64;
                    //} else {
                    //    score *= (i - lx) as f64;
                    //}

                    score *= dx as f64;

                    div_pos.push((score, 'X', i));
                }
            }
            if dy >= (1 << p) * 2 {
                for i in ly + (1 << p)..ry - (1 << p) + 1 {
                    let mut score = 0.00001;
                    for j in lx..rx {
                        let mut over = [0.0, 0.0, 0.0, 0.0];
                        let mut under = [0.0, 0.0, 0.0, 0.0];

                        let mut q = 1;
                        for k in 0..(1 << p) {
                            for c in 0..4 {
                                over[c] += png[i + k][j][c] as f64;
                                under[c] += png[i - 1 - k][j][c] as f64;
                            }
                            if k == q - 1 {
                                let mut tmp = 0.0;
                                for c in 0..4 {
                                    tmp += (over[c] - under[c]) * (over[c] - under[c]);
                                }
                                if k == 0 {
                                    tmp *= 25.0;
                                }
                                q *= 2;
                                score += tmp.sqrt();
                            }
                        }
                    }

                    //if ry - i < i - ly {
                    //    score *= (ry - i) as f64;
                    //} else {
                    //    score *= (i - ly) as f64;
                    //}
                    score *= dy as f64;

                    div_pos.push((score, 'Y', i));
                }
            }
        }

        div_pos.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        list[now].target = 64;
        //list[now].target = 3;
        //while list[now].target * list[now].target < dx * dy / 50 {
        //    list[now].target += 1;
        //}
        if list[now].target > div_pos.len() {
            list[now].target = div_pos.len();
        }

        for i in 0..list[now].target {
            let mut div = div_pos[i];
            let left;
            let right;
            if div.1 == 'X' {
                left = (9999999999.0, [ly, ry, lx, div.2]);
                right = (9999999999.0, [ly, ry, div.2, rx]);
            } else {
                left = (9999999999.0, [ly, div.2, lx, rx]);
                right = (9999999999.0, [div.2, ry, lx, rx]);
            }
            div.0 = 9999999999.0;
            list[now].next.push((div, left, right));
        }
    }

    if list[now].next.len() == 0 {
        /*
        eprintln!(
            "?? cnt:{} ({},{})-({},{}):{}",
            list[now].cnt, ly, lx, ry, rx, list[now].best
        );
        */
        return list[now].best;
    }

    let choice;
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

    let left = list[now].next[choice].1;
    let right = list[now].next[choice].2;

    let leftret = search(left.1[0], left.1[1], left.1[2], left.1[3], map, list, png);
    let rightret = search(
        right.1[0], right.1[1], right.1[2], right.1[3], map, list, png,
    );

    let s = {
        let s1 = (400.001 - right.1[0] as f64) * (400.001 - right.1[2] as f64);
        let s2 = (400.001 - left.1[0] as f64) * (400.001 - left.1[2] as f64) - s1;
        if s1 > s2 {
            s1
        } else {
            s2
        }
    };

    let ret = leftret
        + rightret
        + (400.0 * 400.0 / s) * 1.0
        + (400.0 * 400.0 / ((400.00001 - lx as f64) * (400.00001 - ly as f64))) * 7.0;

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
            leftret,
            rightret,
            list[now].def
        );
    }
    return list[now].best;
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
    next: Vec<((f64, char, usize), (f64, [usize; 4]), (f64, [usize; 4]))>,
}
