use crate::wata::*;
use crate::*;
use crate::{BlockId, Move};
//use std::collections::HashMap;

//png       画像データ
//border    1行のエラーの合計許容値。3~10くらいがおススメ
//combo     連続何行でswap対象になるか。10くらいがお勧め
pub fn solve_swap(png: &mut Vec<Vec<[u8; 4]>>, border: f64, combo: usize) -> (f64, Program) {
    let campus_size = 400 as usize;

    let mut target_color_x = [[0, 0, 0, 0]; 400];
    for k in 0..4 {
        for x in 0..campus_size {
            let mut tmp_v = vec![];
            for y in 0..campus_size {
                tmp_v.push(png[y][x][k]);
            }
            tmp_v.sort();
            target_color_x[x][k] = tmp_v[tmp_v.len() / 2];
        }
    }

    let mut sum_x = vec![0.0; campus_size];
    for y in 0..campus_size {
        for x in 0..campus_size {
            sum_x[x] += get_diff(target_color_x[x], png[y][x]);
        }
    }

    let mut last = campus_size;
    while last > 0 {
        if sum_x[last - 1] <= border {
            last -= 1;
        } else {
            break;
        }
    }

    let mut xlist: Vec<[usize; 3]> = vec![];

    let mut find = 0;
    loop {
        let mut flag = false;
        let mut bestpos = 9999;
        let mut bestfind = 0;

        for x in 0..last {
            if sum_x[x] <= border {
                find += 1;
            } else {
                if find >= combo {
                    if diff(last / 2, x) < diff(last / 2, bestpos) {
                        bestpos = x;
                        bestfind = find;
                    }
                    flag = true;
                    find = 0;
                } else {
                    find = 0;
                }
            }
        }
        if !flag {
            break;
        }

        if bestpos != 9999 {
            let mut diff = bestpos;
            if diff > last - bestpos {
                diff = last - bestpos;
            }
            xlist.push([bestpos - diff, last - diff, diff]);
            swap_x(bestpos - diff, last - diff, diff, png, &mut sum_x);
            last -= bestfind;
        }
    }

    let mut target_color_y = [[0, 0, 0, 0]; 400];
    for k in 0..4 {
        for y in 0..campus_size {
            let mut tmp_v = vec![];
            for x in 0..campus_size {
                tmp_v.push(png[y][x][k]);
            }
            tmp_v.sort();
            target_color_y[y][k] = tmp_v[tmp_v.len() / 2];
        }
    }

    let mut sum_y = vec![0.0; campus_size];
    for y in 0..campus_size {
        for x in 0..campus_size {
            sum_y[y] += get_diff(target_color_y[y], png[y][x]);
        }
    }

    let mut last = campus_size;
    while last > 0 {
        if sum_y[last - 1] <= border {
            last -= 1;
        } else {
            break;
        }
    }

    let mut ylist: Vec<[usize; 3]> = vec![];

    let mut find = 0;
    loop {
        let mut flag = false;
        let mut bestpos = 9999;
        let mut bestfind = 0;

        for y in 0..last {
            if sum_y[y] <= border {
                find += 1;
            } else {
                if find >= combo {
                    if diff(last / 2, y) < diff(last / 2, bestpos) {
                        bestpos = y;
                        bestfind = find;
                    }
                    flag = true;
                    find = 0;
                } else {
                    find = 0;
                }
            }
        }
        if !flag {
            break;
        }

        if bestpos != 9999 {
            let mut diff = bestpos;
            if diff > last - bestpos {
                diff = last - bestpos;
            }
            ylist.push([bestpos - diff, last - diff, diff]);
            swap_y(bestpos - diff, last - diff, diff, png, &mut sum_y);
            last -= bestfind;
        }
    }

    eprintln!("xswap:{}, yswap:{}", xlist.len(), ylist.len());

    let (_, moves) = solve2(&png);

    let mut blocks = vec![BlockId(vec![0])];
    let mut id = 0;

    let mut ans: Vec<Move> = vec![];

    for p in moves {
        //score += p.score();
        ans.push(p.clone());
        //println!("{}", p.clone());

        match p {
            Move::LineCut(_, _, _) => {
                let block = blocks.pop().unwrap();
                blocks.extend(block.cut());
            }
            Move::Merge(_, _) => {
                let _ = blocks.pop().unwrap();
                let _ = blocks.pop().unwrap();
                id += 1;
                blocks.push(BlockId(vec![id]));
            }
            _ => {}
        }
    }

    if xlist.len() + ylist.len() >= 1 {
        let mut moves2: Vec<Move> = vec![];

        while blocks.len() >= 2 {
            let b1 = blocks.pop().unwrap();
            let b2 = blocks.pop().unwrap();
            moves2.push(Move::Merge(b1, b2));
            id += 1;
            blocks.push(BlockId(vec![id]));
        }

        xlist.reverse();

        for x in xlist {
            {
                let mut p1 = 1;
                let mut p2 = 0;

                if x[0] != 0 {
                    let block = blocks.pop().unwrap();
                    moves2.push(Move::LineCut(block.clone(), 'X', (x[0]) as i32));
                    blocks.extend(block.cut());
                }
                {
                    let block = blocks.pop().unwrap();
                    moves2.push(Move::LineCut(block.clone(), 'X', (x[0] + x[2]) as i32));
                    blocks.extend(block.cut());
                }
                if x[0] + x[2] != x[1] {
                    let block = blocks.pop().unwrap();
                    moves2.push(Move::LineCut(block.clone(), 'X', (x[1]) as i32));
                    blocks.extend(block.cut());
                    p1 += 1;
                }
                if x[1] + x[2] != 400 {
                    let block = blocks.pop().unwrap();
                    moves2.push(Move::LineCut(block.clone(), 'X', (x[1] + x[2]) as i32));
                    blocks.extend(block.cut());
                    p1 += 1;
                    p2 += 1;
                }

                let le = blocks.len();

                let mut v: Vec<BlockId> = vec![];
                for _ in 0..le {
                    let block = blocks.pop().unwrap();
                    v.push(block);
                }

                moves2.push(Move::Swap(v[p2].clone(), v[p1].clone()));

                for i in 0..le {
                    if i == p1 {
                        blocks.push(v[p1].clone());
                    } else if i == p2 {
                        blocks.push(v[p2].clone());
                    } else {
                        blocks.push(v[i].clone());
                    }
                }

                while blocks.len() >= 2 {
                    let b1 = blocks.pop().unwrap();
                    let b2 = blocks.pop().unwrap();
                    moves2.push(Move::Merge(b1, b2));
                    id += 1;
                    blocks.push(BlockId(vec![id]));
                }
            }
        }

        ylist.reverse();

        for y in ylist {
            let mut p1 = 1;
            let mut p2 = 0;
            if y[0] != 0 {
                let block = blocks.pop().unwrap();
                moves2.push(Move::LineCut(block.clone(), 'Y', (y[0]) as i32));
                blocks.extend(block.cut());
            }
            {
                let block = blocks.pop().unwrap();
                moves2.push(Move::LineCut(block.clone(), 'Y', (y[0] + y[2]) as i32));
                blocks.extend(block.cut());
            }
            if y[0] + y[2] != y[1] {
                let block = blocks.pop().unwrap();
                moves2.push(Move::LineCut(block.clone(), 'Y', (y[1]) as i32));
                blocks.extend(block.cut());
                p1 += 1;
            }
            if y[1] + y[2] != 400 {
                let block = blocks.pop().unwrap();
                moves2.push(Move::LineCut(block.clone(), 'Y', (y[1] + y[2]) as i32));
                blocks.extend(block.cut());
                p1 += 1;
                p2 += 1;
            }

            let le = blocks.len();

            let mut v: Vec<BlockId> = vec![];
            for _ in 0..le {
                let block = blocks.pop().unwrap();
                v.push(block);
            }

            moves2.push(Move::Swap(v[p2].clone(), v[p1].clone()));

            for i in 0..le {
                if i == p1 {
                    blocks.push(v[p1].clone());
                } else if i == p2 {
                    blocks.push(v[p2].clone());
                } else {
                    blocks.push(v[i].clone());
                }
            }

            while blocks.len() >= 2 {
                let b1 = blocks.pop().unwrap();
                let b2 = blocks.pop().unwrap();
                moves2.push(Move::Merge(b1, b2));
                id += 1;
                blocks.push(BlockId(vec![id]));
            }
        }

        for p in moves2 {
            ans.push(p.clone());
            //score += p.score();
        }
    }
    (0.0, ans)
}

fn diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}

fn swap_x(lx1: usize, lx2: usize, diff: usize, png: &mut Vec<Vec<[u8; 4]>>, sum: &mut Vec<f64>) {
    for x in 0..diff {
        for y in 0..400 {
            let num = png[y][lx1 + x];
            png[y][lx1 + x] = png[y][lx2 + x];
            png[y][lx2 + x] = num;
        }
        let sum_tmp = sum[lx1 + x];
        sum[lx1 + x] = sum[lx2 + x];
        sum[lx2 + x] = sum_tmp;
    }
    return;
}

fn swap_y(ly1: usize, ly2: usize, diff: usize, png: &mut Vec<Vec<[u8; 4]>>, sum: &mut Vec<f64>) {
    for y in 0..diff {
        for x in 0..400 {
            let num = png[ly1 + y][x];
            png[ly1 + y][x] = png[ly2 + y][x];
            png[ly2 + y][x] = num;
        }
        let sum_tmp = sum[ly1 + y];
        sum[ly1 + y] = sum[ly2 + y];
        sum[ly2 + y] = sum_tmp;
    }
    return;
}

fn get_diff(a: [u8; 4], b: [u8; 4]) -> f64 {
    f64::sqrt(p2(a[0], b[0]) + p2(a[1], b[1]) + p2(a[2], b[2]) + p2(a[3], b[3])) * 0.005
}

fn p2(a: u8, b: u8) -> f64 {
    (a as f64 - b as f64) * (a as f64 - b as f64)
}
