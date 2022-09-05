#![allow(non_snake_case)]

use crate::{color::MedianColorSelector, *};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
pub struct Median {
    que1: BinaryHeap<u8>,
    que2: BinaryHeap<u8>,
    sum1: u32,
    sum2: u32,
}

impl Median {
    pub fn new() -> Self {
        Self {
            que1: BinaryHeap::new(),
            que2: BinaryHeap::new(),
            sum1: 0,
            sum2: 0,
        }
    }
    pub fn push(&mut self, v: u8) {
        if self.que1.len() <= self.que2.len() {
            self.que1.push(v);
            self.sum1 += v as u32;
        } else {
            self.que2.push(!v);
            self.sum2 += v as u32;
        }
        if self.que2.len() > 0 && *self.que1.peek().unwrap() > !self.que2.peek().unwrap() {
            let v1 = self.que1.pop().unwrap();
            let v2 = !self.que2.pop().unwrap();
            self.que1.push(v2);
            self.que2.push(!v1);
            self.sum1 -= v1 as u32;
            self.sum1 += v2 as u32;
            self.sum2 -= v2 as u32;
            self.sum2 += v1 as u32;
        }
    }
    pub fn get(&self) -> (u8, u32) {
        let v = *self.que1.peek().unwrap();
        let diff1 = v as u32 * self.que1.len() as u32 - self.sum1;
        let diff2 = self.sum2 - v as u32 * self.que2.len() as u32;
        (v, diff1 + diff2)
    }
}

pub const INF: f64 = 1e10;
pub static MAX_WIDTH: Lazy<usize> = Lazy::new(|| {
    std::env::var("MAX_WIDTH")
        .unwrap_or("40".to_owned())
        .parse()
        .unwrap()
});
pub static MAX_AREA: Lazy<usize> = Lazy::new(|| {
    std::env::var("MAX_AREA")
        .unwrap_or("5000".to_owned())
        .parse()
        .unwrap()
});

pub fn solve(png: &Vec<Vec<[u8; 4]>>) -> (f64, Program) {
    let h = png.len();
    let w = png[0].len();
    let mut cand_x = vec![];
    for lx in 0..w {
        for ux in lx + 1..=w {
            if ux - lx <= 100 && (ux == 0 || ux - lx <= *MAX_WIDTH || ux == w) {
                cand_x.push((lx, ux));
            }
        }
    }
    let mut dp_y = mat![(INF, vec![]); w; w + 1];
    let mut tmp = vec![];
    let bar = indicatif::ProgressBar::new(cand_x.len() as u64);
    cand_x
        .into_par_iter()
        .map(|(lx, ux)| {
            bar.inc(1);
            let mut dp = vec![(INF, !0, [0; 4]); h + 1];
            dp[0].0 = 0.0;
            for ly in 0..h {
                let mut median = vec![Median::new(); 4];
                for y in ly..h {
                    for x in lx..ux {
                        for c in 0..4 {
                            median[c].push(png[y][x][c]);
                        }
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = [0; 4];
                    let mut cost1 = 0;
                    for c in 0..4 {
                        let (a, diff) = median[c].get();
                        color[c] = a;
                        cost1 += diff;
                    }
                    let mut cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    if y + 1 != h {
                        cost2 += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                        let dy = (h - y - 1).max(y + 1 - ly);
                        cost2 += (1.0 * (w * h) as f64 / ((w - lx) * dy) as f64).round();
                    }
                    let cost = dp[ly].0 + cost1 as f64 * 0.005 * 0.5 + cost2;
                    if dp[y + 1].0.setmin(cost) {
                        dp[y + 1].1 = ly;
                        dp[y + 1].2 = color;
                    }
                }
            }
            let mut cost = 0.0;
            let mut out = vec![];
            let mut y = h;
            while y > 0 {
                let (color, diff) = color::best_color2(png, lx, ux, dp[y].1, y);
                cost += diff * 0.005;
                cost += (5.0 * (w * h) as f64 / ((w - lx) * (h - dp[y].1)) as f64).round();
                if y != h {
                    cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - dp[y].1)) as f64).round();
                    let dy = (h - y).max(y - dp[y].1);
                    cost += (1.0 * (w * h) as f64 / ((w - lx) * dy) as f64).round();
                }
                out.push((y, color));
                y = dp[y].1;
            }
            out.reverse();
            (lx, ux, (cost, out))
        })
        .collect_into_vec(&mut tmp);
    bar.finish();
    for (lx, ux, ret) in tmp {
        dp_y[lx][ux] = ret;
    }
    let mut dp_x = vec![(INF, !0); w + 1];
    dp_x[0].0 = 0.0;
    for lx in 0..w {
        for ux in lx + 1..=w {
            if dp_y[lx][ux].0 < INF {
                let mut cost = dp_x[lx].0 + dp_y[lx][ux].0;
                if ux < w {
                    cost += (7.0 * (w * h) as f64 / ((w - lx) * h) as f64).round();
                }
                if dp_x[ux].0.setmin(cost) {
                    dp_x[ux].1 = lx;
                }
            }
        }
    }
    eprintln!("cost = {}", dp_x[w].0);
    let mut xs = vec![];
    let mut x = w;
    while x > 0 {
        xs.push(x);
        x = dp_x[x].1;
    }
    xs.push(0);
    xs.reverse();

    let mut out = vec![];
    let mut id = 0;
    let mut blocks = vec![BlockId(vec![0])];
    if let Ok(hiding) = std::env::var("HIDING") {
        // 潜伏(8+2000*hiding)
        let hiding = hiding.parse::<usize>().unwrap();
        out.push(Move::LineCut(BlockId(vec![0]), 'y', 1));
        for _ in 0..hiding {
            out.push(Move::Color(BlockId(vec![0, 0]), [0, 0, 0, 0]));
        }
        out.push(Move::Merge(BlockId(vec![0, 0]), BlockId(vec![0, 1])));
        id = 1;
        blocks = vec![BlockId(vec![1])];
    }

    for i in 0..xs.len() - 1 {
        let lx = xs[i];
        let ux = xs[i + 1];
        for &(y, color) in &dp_y[lx][ux].1 {
            let block = blocks.pop().unwrap();
            out.push(Move::Color(block.clone(), color));
            if y < h {
                out.push(Move::LineCut(block.clone(), 'y', y as i32));
                blocks.extend(block.cut());
            } else {
                blocks.push(block);
            }
        }
        while blocks.len() > 1 {
            let b1 = blocks.pop().unwrap();
            let b2 = blocks.pop().unwrap();
            out.push(Move::Merge(b1, b2));
            id += 1;
            blocks.push(BlockId(vec![id]));
        }
        if ux < w {
            let block = blocks.pop().unwrap();
            out.push(Move::LineCut(block.clone(), 'x', ux as i32));
            blocks.push(block.cut()[1].clone());
        }
    }
    (dp_x[w].0, out)
}

pub static HEAVY: Lazy<usize> = Lazy::new(|| {
    std::env::var("HEAVY")
        .unwrap_or("0".to_owned())
        .parse()
        .unwrap()
});

pub fn solve2(png: &Vec<Vec<[u8; 4]>>) -> (f64, Program) {
    solve3(png, &Canvas::new(png[0].len(), png.len()))
}

pub fn solve3(png: &Vec<Vec<[u8; 4]>>, init_canvas: &Canvas) -> (f64, Program) {
    let h = png.len();
    let w = png[0].len();
    let mut cand_x = vec![];
    for lx in 0..w {
        for ux in lx + 1..=w {
            if ux - lx <= 100 && (ux == 0 || ux - lx <= *MAX_WIDTH || ux == w) {
                cand_x.push((lx, ux));
            }
        }
    }
    let mut cand_y = vec![];
    for ly in 0..h {
        for uy in ly + 1..=h {
            if uy - ly <= 100 && (uy == 0 || uy - ly <= *MAX_WIDTH || uy == h) {
                cand_y.push((ly, uy));
            }
        }
    }
    let bar = indicatif::ProgressBar::new(cand_x.len() as u64 + cand_y.len() as u64);
    let mut dp_x = mat![vec![]; h; h + 1];
    let mut dp_y = mat![vec![]; w; w + 1];
    let mut tmp = vec![];
    cand_x
        .into_par_iter()
        .map(|(lx, ux)| {
            bar.inc(1);
            let mut dp = vec![(INF, !0, [0; 4]); h + 1];
            dp[h].0 = 0.0;
            for uy in (1..=h).rev() {
                let mut median = vec![Median::new(); 4];
                for ly in (0..uy).rev() {
                    for x in lx..ux {
                        for c in 0..4 {
                            median[c].push(png[ly][x][c]);
                        }
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = [0; 4];
                    let mut cost1 = 0;
                    for c in 0..4 {
                        let (a, diff) = median[c].get();
                        color[c] = a;
                        cost1 += diff;
                    }
                    let mut cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    if uy != h {
                        cost2 += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                        let dy = (uy - ly).max(h - uy);
                        cost2 += (1.0 * (w * h) as f64 / ((w - lx) * dy) as f64).round();
                    }
                    let cost = dp[uy].0 + cost1 as f64 * 0.005 * 0.5 + cost2;
                    if dp[ly].0.setmin(cost) {
                        dp[ly].1 = uy;
                        dp[ly].2 = color;
                    }
                }
                // TODO: ここでいずれかの復元パスに含まれる矩形について、色とコストを最適化したほうが良いかも？
            }
            (lx, ux, dp)
        })
        .collect_into_vec(&mut tmp);
    for (lx, ux, ret) in tmp {
        dp_y[lx][ux] = ret;
    }
    let mut tmp = vec![];
    cand_y
        .into_par_iter()
        .map(|(ly, uy)| {
            bar.inc(1);
            let mut dp = vec![(INF, !0, [0; 4]); w + 1];
            dp[w].0 = 0.0;
            for ux in (1..=w).rev() {
                let mut median = vec![Median::new(); 4];
                for lx in (0..ux).rev() {
                    for y in ly..uy {
                        for c in 0..4 {
                            median[c].push(png[y][lx][c]);
                        }
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = [0; 4];
                    let mut cost1 = 0;
                    for c in 0..4 {
                        let (a, diff) = median[c].get();
                        color[c] = a;
                        cost1 += diff;
                    }
                    let mut cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    if ux != w {
                        cost2 += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                        let dx = (ux - lx).max(w - ux);
                        cost2 += (1.0 * (w * h) as f64 / (dx * (h - ly)) as f64).round();
                    }
                    let cost = dp[ux].0 + cost1 as f64 * 0.005 * 0.5 + cost2;
                    if dp[lx].0.setmin(cost) {
                        dp[lx].1 = ux;
                        dp[lx].2 = color;
                    }
                }
                // TODO: ここでいずれかの復元パスに含まれる矩形について、色とコストを最適化したほうが良いかも？
            }
            (ly, uy, dp)
        })
        .collect_into_vec(&mut tmp);
    bar.finish();
    for (ly, uy, ret) in tmp {
        dp_x[ly][uy] = ret;
    }
    let mut dp_xy = mat![(INF, (!0, !0)); h + 1; w + 1];
    dp_xy[0][0].0 = 0.0;
    for lx in 0..w {
        for ly in 0..h {
            for ux in lx + 1..=w {
                if dp_y[lx][ux].len() > 0 {
                    let mut cost = dp_xy[lx][ly].0 + dp_y[lx][ux][ly].0;
                    if ux < w {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    }
                    if dp_xy[ux][ly].0.setmin(cost) {
                        dp_xy[ux][ly].1 = (lx, ly);
                    }
                }
            }
            for uy in ly + 1..=h {
                if dp_x[ly][uy].len() > 0 {
                    let mut cost = dp_xy[lx][ly].0 + dp_x[ly][uy][lx].0;
                    if uy < h {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    }
                    if dp_xy[lx][uy].0.setmin(cost) {
                        dp_xy[lx][uy].1 = (lx, ly);
                    }
                }
            }
        }
    }
    let mut tx = w;
    let mut ty = h;
    for x in 0..=w {
        if dp_xy[tx][ty] > dp_xy[x][h] {
            tx = x;
            ty = h;
        }
    }
    for y in 0..=h {
        if dp_xy[tx][ty] > dp_xy[w][y] {
            tx = w;
            ty = y;
        }
    }
    eprintln!("cost = {}", dp_xy[tx][ty].0);
    let mut xys = vec![];
    {
        let mut x = tx;
        let mut y = ty;
        while x > 0 || y > 0 {
            xys.push((x, y));
            let (x2, y2) = dp_xy[x][y].1;
            x = x2;
            y = y2;
        }
        xys.push((0, 0));
        xys.reverse();
    }
    // dbg!(&xys);
    let mut dp1 = vec![];
    let bar = indicatif::ProgressBar::new(xys.len() as u64 - 1);
    (0..xys.len() - 1)
        .into_par_iter()
        .map(|i| {
            let (lx, ly) = xys[i];
            let (ux, uy) = xys[i + 1];
            let ret = if ly == uy {
                if uy - ly < *HEAVY {
                    heavy_dp_y(png, lx, ux, ly)
                } else {
                    let mut tmp = vec![];
                    let mut y = ly;
                    while y < h {
                        tmp.push((dp_y[lx][ux][y].1, dp_y[lx][ux][y].2));
                        y = dp_y[lx][ux][y].1;
                    }
                    (dp_y[lx][ux][ly].0, tmp)
                }
            } else {
                if ux - lx < *HEAVY {
                    heavy_dp_x(png, ly, uy, lx)
                } else {
                    let mut tmp = vec![];
                    let mut x = lx;
                    while x < w {
                        tmp.push((dp_x[ly][uy][x].1, dp_x[ly][uy][x].2));
                        x = dp_x[ly][uy][x].1;
                    }
                    (dp_x[ly][uy][lx].0, tmp)
                }
            };
            bar.inc(1);
            ret
        })
        .collect_into_vec(&mut dp1);
    bar.finish();

    let (mut id, mut out) = all_merge(&init_canvas);
    let mut blocks = vec![BlockId(vec![id])];
    let mut score = dp_xy[tx][ty].0;
    for i in 0..xys.len() - 1 {
        let (lx, ly) = xys[i];
        let (ux, uy) = xys[i + 1];
        if ly == uy {
            score += dp1[i].0 - dp_y[lx][ux][ly].0;
            for &(y, color) in &dp1[i].1 {
                let block = blocks.pop().unwrap();
                out.push(Move::Color(block.clone(), color));
                if y < h {
                    out.push(Move::LineCut(block.clone(), 'y', y as i32));
                    blocks.extend(block.cut());
                } else {
                    blocks.push(block);
                }
            }
            while blocks.len() > 1 {
                let b1 = blocks.pop().unwrap();
                let b2 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                id += 1;
                blocks.push(BlockId(vec![id]));
            }
            if ux < w {
                let block = blocks.pop().unwrap();
                out.push(Move::LineCut(block.clone(), 'x', ux as i32));
                blocks.push(block.cut()[1].clone());
            }
        } else {
            score += dp1[i].0 - dp_x[ly][uy][lx].0;
            for &(x, color) in &dp1[i].1 {
                let block = blocks.pop().unwrap();
                out.push(Move::Color(block.clone(), color));
                if x < w {
                    out.push(Move::LineCut(block.clone(), 'x', x as i32));
                    blocks.extend(block.cut());
                } else {
                    blocks.push(block);
                }
            }
            while blocks.len() > 1 {
                let b1 = blocks.pop().unwrap();
                let b2 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                id += 1;
                blocks.push(BlockId(vec![id]));
            }
            if uy < h {
                let block = blocks.pop().unwrap();
                out.push(Move::LineCut(block.clone(), 'y', uy as i32));
                blocks.push(block.cut()[1].clone());
            }
        }
    }
    let mut canvas = init_canvas.clone();
    let mut cost = canvas.apply_all(out.clone());
    cost += similarity(png, &canvas.bitmap);
    eprintln!("expected = {}", score);
    eprintln!("actual = {}", cost);
    (cost, out)
}

pub fn heavy_dp_x(
    png: &Vec<Vec<[u8; 4]>>,
    ly: usize,
    uy: usize,
    x0: usize,
) -> (f64, Vec<(usize, [u8; 4])>) {
    let h = png.len();
    let w = png[0].len();
    let mut dp = vec![(INF, !0, [0; 4]); w + 1];
    dp[w].0 = 0.0;
    for ux in (x0 + 1..=w).rev() {
        let mut tmp = vec![];
        (x0..ux)
            .into_par_iter()
            .map(|lx| color::best_color2(png, lx, ux, ly, uy))
            .collect_into_vec(&mut tmp);
        for lx in (x0..ux).rev() {
            let (color, cost1) = tmp[lx - x0];
            let mut cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
            if ux != w {
                cost2 += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                let dx = (ux - lx).max(w - ux);
                cost2 += (1.0 * (w * h) as f64 / (dx * (h - ly)) as f64).round();
            }
            let cost = dp[ux].0 + cost1 as f64 * 0.005 + cost2;
            if dp[lx].0.setmin(cost) {
                dp[lx].1 = ux;
                dp[lx].2 = color;
            }
        }
    }
    let mut out = vec![];
    let mut x = x0;
    while x < w {
        out.push((dp[x].1, dp[x].2));
        x = dp[x].1;
    }
    (dp[x0].0, out)
}

pub fn heavy_dp_y(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    ux: usize,
    y0: usize,
) -> (f64, Vec<(usize, [u8; 4])>) {
    let h = png.len();
    let w = png[0].len();
    let mut dp = vec![(INF, !0, [0; 4]); h + 1];
    dp[h].0 = 0.0;
    for uy in (y0 + 1..=h).rev() {
        let mut tmp = vec![];
        (y0..uy)
            .into_par_iter()
            .map(|ly| color::best_color2(png, lx, ux, ly, uy))
            .collect_into_vec(&mut tmp);

        for ly in (y0..uy).rev() {
            let (color, cost1) = tmp[ly - y0];
            let mut cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
            if uy != h {
                cost2 += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                let dy = (uy - ly).max(h - uy);
                cost2 += (1.0 * (w * h) as f64 / ((w - lx) * dy) as f64).round();
            }
            let cost = dp[uy].0 + cost1 as f64 * 0.005 + cost2;
            if dp[ly].0.setmin(cost) {
                dp[ly].1 = uy;
                dp[ly].2 = color;
            }
        }
    }
    let mut out = vec![];
    let mut y = y0;
    while y < h {
        out.push((dp[y].1, dp[y].2));
        y = dp[y].1;
    }
    (dp[y0].0, out)
}

pub fn all_merge(canvas: &Canvas) -> (u32, Vec<Move>) {
    let mut id = canvas.blocks.keys().map(|b| b.0[0]).max().unwrap();
    let mut out = vec![];
    let mut bs = canvas
        .blocks
        .iter()
        .map(|(id, block)| (block.clone(), id.clone()))
        .collect::<Vec<_>>();
    bs.sort();
    let mut stack: Vec<(Block, BlockId)> = vec![];
    for b in bs {
        stack.push(b);
        while stack.len() >= 2
            && canvas::check_merge_compatibility(
                &stack[stack.len() - 2].0,
                &stack[stack.len() - 1].0,
            )
            .is_ok()
        {
            let b2 = stack.pop().unwrap();
            let b1 = stack.pop().unwrap();
            out.push(Move::Merge(b1.1, b2.1));
            id += 1;
            stack.push((Block(b1.0 .0, b2.0 .1), BlockId(vec![id])));
        }
    }
    assert_eq!(stack.len(), 1);
    (id, out)
}

const MUL: f64 = 0.666;

pub fn solve4(png: &Vec<Vec<[u8; 4]>>, init_canvas: &Canvas) -> (f64, Program) {
    let D = *MAX_WIDTH;
    let h = png.len();
    let w = png[0].len();

    // 小さな矩形の処理
    // (cost, prev_type (0:x, 1:y, 2:一色で塗る), prev_val)
    // dp[x][dx][y][dy] := [x,w)×[y,h)をキャンバスとして、閉区間[x,x+dx]×[y,y+dy]を塗り、かつmergeで後処理をする最小コスト
    let mut dp = mat![(INF, (0u32, 0u32)); w; D; h; D];

    // 一色で塗る場合の計算: O(whD^3)
    let mut cand = vec![];
    for x in 0..w {
        for dx in 0..D {
            if x + dx < w {
                cand.push((x, dx));
            }
        }
    }
    let bar = indicatif::ProgressBar::new(cand.len() as u64);
    let mut tmp = vec![];
    cand.into_par_iter()
        .map(|(lx, dx)| {
            bar.inc(1);
            let ux = lx + dx + 1;
            let mut dp = mat![(INF, (0u32, 0u32)); h; D];
            for ly in 0..h {
                let mut median = vec![Median::new(); 4];
                for uy in ly..h.min(ly + D) {
                    for x in lx..ux {
                        for c in 0..4 {
                            median[c].push(png[uy][x][c]);
                        }
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = 0;
                    let mut cost1 = 0;
                    for c in 0..4 {
                        let (a, diff) = median[c].get();
                        color |= (a as u32) << (c * 8);
                        cost1 += diff;
                    }
                    let cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    dp[ly][uy - ly] = (cost1 as f64 * 0.005 * MUL + cost2, (2, color));
                }
            }
            (lx, dx, dp)
        })
        .collect_into_vec(&mut tmp);
    bar.finish();
    for (lx, dx, ret) in tmp {
        dp[lx][dx] = ret;
    }

    // 分割する場合: O(whD^3)
    let bar = indicatif::ProgressBar::new((D * D) as u64);
    for dx in 0..D {
        for dy in 0..D {
            bar.inc(1);
            let mut cand = vec![];
            for x in 0..w - dx {
                for y in 0..h - dy {
                    cand.push((x, y));
                }
            }
            let mut tmp = vec![];
            cand.par_iter()
                .map(|&(x, y)| {
                    let mut ret = (INF, (0u32, 0u32));
                    for x2 in x + 1..=x + dx {
                        let mut cost = dp[x][x2 - x - 1][y][dy].0 + dp[x2][x + dx - x2][y][dy].0;
                        cost += (7.0 * (w * h) as f64 / ((w - x) * (h - y)) as f64).round();
                        if y + dy + 1 < h {
                            cost += (1.0 * (w * h) as f64
                                / ((w - x2).max(x2 - x) * (h - y)) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (0, x2 as u32);
                        }
                    }
                    for y2 in y + 1..=y + dy {
                        let mut cost = dp[x][dx][y][y2 - y - 1].0 + dp[x][dx][y2][y + dy - y2].0;
                        cost += (7.0 * (w * h) as f64 / ((w - x) * (h - y)) as f64).round();
                        if x + dx + 1 < w {
                            cost += (1.0 * (w * h) as f64
                                / ((h - y2).max(y2 - y) * (w - x)) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (1, y2 as u32);
                        }
                    }
                    ret
                })
                .collect_into_vec(&mut tmp);
            for ((x, y), ret) in cand.into_iter().zip(tmp) {
                dp[x][dx][y][dy].setmin(ret);
            }
        }
    }
    bar.finish();

    // 右端がw or 上端がhで固定の細長い矩形の処理
    // dp_x[ly][uy][lx] := [lx,w)×[ly,h)をキャンバスとして、[lx,w)×[ly,uy)を塗り、かつmergeで後処理をする最小コスト
    // dp_y[lx][ux][ly] := [lx,w)×[ly,h)をキャンバスとして、[lx,ux)×[ly,h)を塗り、かつmergeで後処理をする最小コスト
    let mut dp_x = mat![vec![]; h; h + 1];
    let mut dp_y = mat![vec![]; w; w + 1];
    let mut cand_x = vec![];
    for lx in 0..w {
        for ux in lx + 1..=w {
            if ux - lx <= 100 && (ux == 0 || ux - lx <= *MAX_WIDTH || ux == w) {
                cand_x.push((lx, ux));
            }
        }
    }
    let mut cand_y = vec![];
    for ly in 0..h {
        for uy in ly + 1..=h {
            if uy - ly <= 100 && (uy == 0 || uy - ly <= *MAX_WIDTH || uy == h) {
                cand_y.push((ly, uy));
            }
        }
    }
    let bar = indicatif::ProgressBar::new(cand_x.len() as u64 + cand_y.len() as u64);
    let mut tmp = vec![];
    let A = *MAX_AREA;
    cand_x
        .into_par_iter()
        .map(|(lx, ux)| {
            bar.inc(1);
            let mut ret = vec![(INF, !0, None); h + 1];
            ret[h].0 = 0.0;
            for uy in (1..=h).rev() {
                let mut median = vec![Median::new(); 4];
                for ly in (0..uy).rev() {
                    if uy - ly > D && ux < w && uy < h && (uy - ly) * (ux - lx) > A {
                        break;
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = [0; 4];
                    let mut cost1 = 0;
                    if ux - lx > D || ux == w || uy == h || D * (ux - lx) <= A {
                        for x in lx..ux {
                            for c in 0..4 {
                                median[c].push(png[ly][x][c]);
                            }
                        }
                        for c in 0..4 {
                            let (a, diff) = median[c].get();
                            color[c] = a;
                            cost1 += diff;
                        }
                    } else {
                        cost1 = 1000000000;
                    }
                    let mut cost = cost1 as f64 * 0.005 * MUL
                        + (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    let color = if ux - lx <= D
                        && uy - ly <= D
                        && cost.setmin(dp[lx][ux - lx - 1][ly][uy - ly - 1].0 as f64)
                    {
                        None
                    } else {
                        Some(color)
                    };
                    cost += ret[uy].0;
                    if uy != h {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                        if ux < w {
                            let dy = (uy - ly).max(h - uy);
                            cost += (1.0 * (w * h) as f64 / ((w - lx) * dy) as f64).round();
                        }
                    }
                    if ret[ly].0.setmin(cost) {
                        ret[ly].1 = uy;
                        ret[ly].2 = color;
                    }
                }
            }
            (lx, ux, ret)
        })
        .collect_into_vec(&mut tmp);
    for (lx, ux, ret) in tmp {
        dp_y[lx][ux] = ret;
    }
    let mut tmp = vec![];
    cand_y
        .into_par_iter()
        .map(|(ly, uy)| {
            bar.inc(1);
            let mut ret = vec![(INF, !0, None); w + 1];
            ret[w].0 = 0.0;
            for ux in (1..=w).rev() {
                let mut median = vec![Median::new(); 4];
                for lx in (0..ux).rev() {
                    if ux - lx > D && ux < w && uy < h && (uy - ly) * (ux - lx) > A {
                        break;
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = [0; 4];
                    let mut cost1 = 0;
                    if uy - ly > D || ux == w || uy == h || D * (uy - ly) <= A {
                        for y in ly..uy {
                            for c in 0..4 {
                                median[c].push(png[y][lx][c]);
                            }
                        }
                        for c in 0..4 {
                            let (a, diff) = median[c].get();
                            color[c] = a;
                            cost1 += diff;
                        }
                    } else {
                        cost1 = 1000000000;
                    }
                    let mut cost = cost1 as f64 * 0.005 * MUL
                        + (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    let color = if ux - lx <= D
                        && uy - ly <= D
                        && cost.setmin(dp[lx][ux - lx - 1][ly][uy - ly - 1].0 as f64)
                    {
                        None
                    } else {
                        Some(color)
                    };
                    cost += ret[ux].0;
                    if ux != w {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                        if uy < h {
                            let dx = (ux - lx).max(w - ux);
                            cost += (1.0 * (w * h) as f64 / ((h - ly) * dx) as f64).round();
                        }
                    }
                    if ret[lx].0.setmin(cost) {
                        ret[lx].1 = ux;
                        ret[lx].2 = color;
                    }
                }
            }
            (ly, uy, ret)
        })
        .collect_into_vec(&mut tmp);
    bar.finish();
    for (ly, uy, ret) in tmp {
        dp_x[ly][uy] = ret;
    }

    // dp_xy[x][y] := [x,w)×[y,h)をキャンバスとして全体を塗る最小コスト
    let mut dp_xy = mat![(INF, (!0, !0)); h + 1; w + 1];
    dp_xy[0][0].0 = 0.0;
    for lx in 0..w {
        for ly in 0..h {
            for ux in lx + 1..=w {
                if dp_y[lx][ux].len() > 0 {
                    let mut cost = dp_xy[lx][ly].0 + dp_y[lx][ux][ly].0;
                    if ux < w {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    }
                    if dp_xy[ux][ly].0.setmin(cost) {
                        dp_xy[ux][ly].1 = (lx, ly);
                    }
                }
            }
            for uy in ly + 1..=h {
                if dp_x[ly][uy].len() > 0 {
                    let mut cost = dp_xy[lx][ly].0 + dp_x[ly][uy][lx].0;
                    if uy < h {
                        cost += (7.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    }
                    if dp_xy[lx][uy].0.setmin(cost) {
                        dp_xy[lx][uy].1 = (lx, ly);
                    }
                }
            }
        }
    }
    let mut tx = w;
    let mut ty = h;
    for x in 0..=w {
        if dp_xy[tx][ty] > dp_xy[x][h] {
            tx = x;
            ty = h;
        }
    }
    for y in 0..=h {
        if dp_xy[tx][ty] > dp_xy[w][y] {
            tx = w;
            ty = y;
        }
    }
    eprintln!("cost = {}", dp_xy[tx][ty].0);
    let mut xys = vec![];
    {
        let mut x = tx;
        let mut y = ty;
        while x > 0 || y > 0 {
            xys.push((x, y));
            let (x2, y2) = dp_xy[x][y].1;
            x = x2;
            y = y2;
        }
        xys.push((0, 0));
        xys.reverse();
    }
    let (mut id, mut out) = all_merge(&init_canvas);
    let mut blocks = vec![BlockId(vec![id])];
    for i in 0..xys.len() - 1 {
        let (lx, ly) = xys[i];
        let (ux, uy) = xys[i + 1];
        if ly == uy {
            let mut y = ly;
            while y < h {
                if let Some(color) = dp_y[lx][ux][y].2 {
                    out.push(Move::Color(blocks.last().unwrap().clone(), color));
                } else {
                    rec(
                        lx,
                        ux,
                        y,
                        dp_y[lx][ux][y].1,
                        w,
                        h,
                        &dp,
                        &mut id,
                        &mut blocks,
                        &mut out,
                        false,
                    );
                }
                y = dp_y[lx][ux][y].1;
                if y < h {
                    let block = blocks.pop().unwrap();
                    out.push(Move::LineCut(block.clone(), 'y', y as i32));
                    blocks.extend(block.cut());
                }
            }
            if ux < w {
                while blocks.len() > 1 {
                    let b1 = blocks.pop().unwrap();
                    let b2 = blocks.pop().unwrap();
                    out.push(Move::Merge(b1, b2));
                    id += 1;
                    blocks.push(BlockId(vec![id]));
                }
                let block = blocks.pop().unwrap();
                out.push(Move::LineCut(block.clone(), 'x', ux as i32));
                blocks.push(block.cut()[1].clone());
            }
        } else {
            let mut x = lx;
            while x < w {
                if let Some(color) = dp_x[ly][uy][x].2 {
                    out.push(Move::Color(blocks.last().unwrap().clone(), color));
                } else {
                    rec(
                        x,
                        dp_x[ly][uy][x].1,
                        ly,
                        uy,
                        w,
                        h,
                        &dp,
                        &mut id,
                        &mut blocks,
                        &mut out,
                        false,
                    );
                }
                x = dp_x[ly][uy][x].1;
                if x < w {
                    let block = blocks.pop().unwrap();
                    out.push(Move::LineCut(block.clone(), 'x', x as i32));
                    blocks.extend(block.cut());
                }
            }
            if uy < h {
                while blocks.len() > 1 {
                    let b1 = blocks.pop().unwrap();
                    let b2 = blocks.pop().unwrap();
                    out.push(Move::Merge(b1, b2));
                    id += 1;
                    blocks.push(BlockId(vec![id]));
                }
                let block = blocks.pop().unwrap();
                out.push(Move::LineCut(block.clone(), 'y', uy as i32));
                blocks.push(block.cut()[1].clone());
            }
        }
    }
    let mut canvas = init_canvas.clone();
    let mut cost = canvas.apply_all(out.clone());
    let mut expected = cost;
    for y in 0..h {
        for x in 0..w {
            for c in 0..4 {
                expected += png[y][x][c].abs_diff(canvas.bitmap[y][x][c]) as f64 * 0.005 * MUL;
            }
        }
    }
    cost += similarity(png, &canvas.bitmap);
    eprintln!("expected = {}", expected);
    eprintln!("actual = {}", cost);
    (cost, out)
}

fn rec(
    lx: usize,
    ux: usize,
    ly: usize,
    uy: usize,
    w: usize,
    h: usize,
    dp: &Vec<Vec<Vec<Vec<(f64, (u32, u32))>>>>,
    id: &mut u32,
    blocks: &mut Vec<BlockId>,
    out: &mut Vec<Move>,
    do_all_merge: bool,
) {
    match dp[lx][ux - lx - 1][ly][uy - ly - 1].1 .0 {
        0 => {
            // eprintln!("{:?}", (lx, ux, ly, uy));
            let x = dp[lx][ux - lx - 1][ly][uy - ly - 1].1 .1 as usize;
            rec(lx, x, ly, uy, w, h, dp, id, blocks, out, do_all_merge);
            let block = blocks.pop().unwrap();
            out.push(Move::LineCut(block.clone(), 'x', x as i32));
            blocks.extend(block.cut());
            rec(x, ux, ly, uy, w, h, dp, id, blocks, out, do_all_merge);
            if do_all_merge || ux < w || uy < h {
                let b2 = blocks.pop().unwrap();
                let b1 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                *id += 1;
                blocks.push(BlockId(vec![*id]));
            }
        }
        1 => {
            // eprintln!("{:?}", (lx, ux, ly, uy));
            let y = dp[lx][ux - lx - 1][ly][uy - ly - 1].1 .1 as usize;
            rec(lx, ux, ly, y, w, h, dp, id, blocks, out, do_all_merge);
            let block = blocks.pop().unwrap();
            out.push(Move::LineCut(block.clone(), 'y', y as i32));
            blocks.extend(block.cut());
            rec(lx, ux, y, uy, w, h, dp, id, blocks, out, do_all_merge);
            if do_all_merge || ux < w || uy < h {
                let b2 = blocks.pop().unwrap();
                let b1 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                *id += 1;
                blocks.push(BlockId(vec![*id]));
            }
        }
        2 => {
            let c = dp[lx][ux - lx - 1][ly][uy - ly - 1].1 .1;
            let color = [
                (c >> 0) as u8,
                (c >> 8) as u8,
                (c >> 16) as u8,
                (c >> 24) as u8,
            ];
            out.push(Move::Color(blocks.last().unwrap().clone(), color));
        }
        _ => unreachable!(),
    }
}

pub static MAX_CANDIDATES: Lazy<usize> = Lazy::new(|| {
    std::env::var("MAX_CANDIDATES")
        .unwrap_or("100".to_owned())
        .parse()
        .unwrap()
});

pub fn solve5(png: &Vec<Vec<[u8; 4]>>, init_canvas: &Canvas, do_all_merge: bool) -> (f64, Program) {
    let D = *MAX_WIDTH;
    let h = png.len();
    let w = png[0].len();
    let CUT = match init_canvas.cost_type {
        CostType::Basic => 7.0,
        CostType::V2 => 2.0,
    };

    // 小さな矩形の処理
    // (cost, prev_type (0:x, 1:y, 2:一色で塗る), prev_val)
    // dp[x][dx][y][dy] := [x,w)×[y,h)をキャンバスとして、閉区間[x,x+dx]×[y,y+dy]を塗り、かつmergeで後処理をする最小コスト
    let mut dp = mat![(INF, (0u32, 0u32)); w; D; h; D];

    // 一色で塗る場合の計算: O(whD^3)
    let mut cand = vec![];
    for x in 0..w {
        for dx in 0..D {
            if x + dx < w {
                cand.push((x, dx));
            }
        }
    }
    let bar = indicatif::ProgressBar::new(cand.len() as u64);
    let mut tmp = vec![];
    cand.into_par_iter()
        .map(|(lx, dx)| {
            bar.inc(1);
            let ux = lx + dx + 1;
            let mut dp = mat![(INF, (0u32, 0u32)); h; D];
            for ly in 0..h {
                let mut median = vec![Median::new(); 4];
                for uy in ly..h.min(ly + D) {
                    for x in lx..ux {
                        for c in 0..4 {
                            median[c].push(png[uy][x][c]);
                        }
                    }
                    // sqrt(r^2+g^2+b^2+a^2)を(|r|+|g|+|b|+|a|)/2に近似
                    let mut color = 0;
                    let mut cost1 = 0;
                    for c in 0..4 {
                        let (a, diff) = median[c].get();
                        color |= (a as u32) << (c * 8);
                        cost1 += diff;
                    }
                    let cost2 = (5.0 * (w * h) as f64 / ((w - lx) * (h - ly)) as f64).round();
                    dp[ly][uy - ly] = (cost1 as f64 * 0.005 * MUL + cost2, (2, color));
                }
            }
            (lx, dx, dp)
        })
        .collect_into_vec(&mut tmp);
    bar.finish();
    for (lx, dx, ret) in tmp {
        dp[lx][dx] = ret;
    }

    // 分割する場合: O(whD^3)
    let bar = indicatif::ProgressBar::new((D * D) as u64);
    for dx in 0..D {
        for dy in 0..D {
            bar.inc(1);
            let mut cand = vec![];
            for x in 0..w - dx {
                for y in 0..h - dy {
                    cand.push((x, y));
                }
            }
            let mut tmp = vec![];
            cand.par_iter()
                .map(|&(x, y)| {
                    let mut ret = (INF, (0u32, 0u32));
                    for x2 in x + 1..=x + dx {
                        let mut cost = dp[x][x2 - x - 1][y][dy].0 + dp[x2][x + dx - x2][y][dy].0;
                        cost += (CUT * (w * h) as f64 / ((w - x) * (h - y)) as f64).round();
                        if do_all_merge || x + dx + 1 < w || y + dy + 1 < h {
                            cost += (1.0 * (w * h) as f64
                                / ((w - x2).max(x2 - x) * (h - y)) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (0, x2 as u32);
                        }
                    }
                    for y2 in y + 1..=y + dy {
                        let mut cost = dp[x][dx][y][y2 - y - 1].0 + dp[x][dx][y2][y + dy - y2].0;
                        cost += (CUT * (w * h) as f64 / ((w - x) * (h - y)) as f64).round();
                        if do_all_merge || x + dx + 1 < w || y + dy + 1 < h {
                            cost += (1.0 * (w * h) as f64
                                / ((h - y2).max(y2 - y) * (w - x)) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (1, y2 as u32);
                        }
                    }
                    ret
                })
                .collect_into_vec(&mut tmp);
            for ((x, y), ret) in cand.into_iter().zip(tmp) {
                dp[x][dx][y][dy].setmin(ret);
            }
        }
    }
    bar.finish();

    let mut weight_x = vec![0; w];
    let mut weight_y = vec![0; w];
    for dx in 1..=D {
        for x in 0..=w - dx {
            for x2 in x + 1..x + dx {
                weight_x[x2] += 1;
            }
        }
    }
    for dy in 1..=D {
        for y in 0..=h - dy {
            for y2 in y + 1..y + dy {
                weight_y[y2] += 1;
            }
        }
    }

    let mut count_x = vec![0i64; w];
    let mut count_y = vec![0i64; h];
    for x in 0..w {
        for dx in 0..D {
            for y in 0..h {
                for dy in 0..D {
                    if dp[x][dx][y][dy].0 < INF {
                        if dp[x][dx][y][dy].1 .0 == 0 {
                            let v = dp[x][dx][y][dy].1 .1 as usize;
                            count_x[v] += 100000000 / weight_x[v];
                        } else if dp[x][dx][y][dy].1 .0 == 1 {
                            let v = dp[x][dx][y][dy].1 .1 as usize;
                            count_y[v] += 100000000 / weight_y[v];
                        }
                    }
                }
            }
        }
    }
    // dbg!(&count_x);
    // ベスト解で使ってる座標を追加するオプションがあっても良いかも
    let mut xs = vec![0, w];
    let mut ys = vec![0, h];
    let mut used_x = vec![false; w];
    let mut used_y = vec![false; h];
    for _ in 0..*MAX_CANDIDATES {
        let v = (1..w)
            .filter(|&v| !used_x[v])
            .max_by_key(|&v| count_x[v])
            .unwrap();
        used_x[v] = true;
        xs.push(v);
        for d in 1..=10 {
            if v > d {
                count_x[v - d] -= count_x[v - d] / (d + 1) as i64;
            }
            if v + d < w {
                count_x[v + d] -= count_x[v + d] / (d + 1) as i64;
            }
        }
    }
    for _ in 0..*MAX_CANDIDATES {
        let v = (1..h)
            .filter(|&v| !used_y[v])
            .max_by_key(|&v| count_y[v])
            .unwrap();
        used_y[v] = true;
        ys.push(v);
        for d in 1..=10 {
            if v > d {
                count_y[v - d] -= count_y[v - d] / (d + 1) as i64;
            }
            if v + d < h {
                count_y[v + d] -= count_y[v + d] / (d + 1) as i64;
            }
        }
    }
    xs.sort();
    ys.sort();
    for i in 0.. {
        if i + 1 >= xs.len() {
            break;
        }
        while xs[i + 1] - xs[i] > D {
            xs.insert(i + 1, xs[i + 1] - D);
        }
    }
    for i in 0.. {
        if i + 1 >= ys.len() {
            break;
        }
        while ys[i + 1] - ys[i] > D {
            ys.insert(i + 1, ys[i + 1] - D);
        }
    }
    // dbg!(&xs);
    // dbg!(&ys);

    let median = MedianColorSelector::new(&png);

    // 候補点を端点とする矩形の処理
    // (cost, prev_type (0:x, 1:y, 2:一色で塗る, 3:dpを参照), prev_val)
    // dp2[lx][ux][ly][uy] := [xs[lx],w)×[ys[ly],h)をキャンバスとして、区間[xs[lx],xs[ux])×[ys[ly],ys[uy])を塗り、かつmergeで後処理をする最小コスト
    let mut dp2 = mat![(INF, (0u32, 0u32)); xs.len(); xs.len(); ys.len(); ys.len()];
    let bar = indicatif::ProgressBar::new((xs.len() * ys.len()) as u64);
    for dx in 1..xs.len() {
        for dy in 1..ys.len() {
            bar.inc(1);
            let mut cand = vec![];
            for lx in 0..xs.len() - dx {
                for ly in 0..ys.len() - dy {
                    cand.push((lx, ly));
                }
            }
            let mut tmp = vec![];
            cand.par_iter()
                .map(|&(lx, ly)| {
                    let ux = lx + dx;
                    let uy = ly + dy;
                    let (color, cost) = median.query(xs[lx], xs[ux], ys[ly], ys[uy]);
                    let cost = cost as f64 * 0.005 * MUL
                        + (5.0 * (w * h) as f64 / ((w - xs[lx]) * (h - ys[ly])) as f64).round();
                    let mut ret = (
                        cost,
                        (
                            2,
                            color[0] as u32
                                | (color[1] as u32) << 8
                                | (color[2] as u32) << 16
                                | (color[3] as u32) << 24,
                        ),
                    );
                    if xs[ux] - xs[lx] <= D && ys[uy] - ys[ly] <= D {
                        if ret
                            .0
                            .setmin(dp[xs[lx]][xs[ux] - xs[lx] - 1][ys[ly]][ys[uy] - ys[ly] - 1].0)
                        {
                            ret.1 = (3, 0);
                        }
                    }
                    for mx in lx + 1..ux {
                        let mut cost = dp2[lx][mx][ly][uy].0 + dp2[mx][ux][ly][uy].0;
                        cost +=
                            (CUT * (w * h) as f64 / ((w - xs[lx]) * (h - ys[ly])) as f64).round();
                        if do_all_merge || ux + 1 < xs.len() || uy + 1 < ys.len() {
                            cost += (1.0 * (w * h) as f64
                                / ((w - xs[mx]).max(xs[mx] - xs[lx]) * (h - ys[ly])) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (0, mx as u32);
                        }
                    }
                    for my in ly + 1..uy {
                        let mut cost = dp2[lx][ux][ly][my].0 + dp2[lx][ux][my][uy].0;
                        cost +=
                            (CUT * (w * h) as f64 / ((w - xs[lx]) * (h - ys[ly])) as f64).round();
                        if do_all_merge || ux + 1 < xs.len() || uy + 1 < ys.len() {
                            cost += (1.0 * (w * h) as f64
                                / ((h - ys[my]).max(ys[my] - ys[ly]) * (w - xs[lx])) as f64)
                                .round();
                        }
                        if ret.0.setmin(cost) {
                            ret.1 = (1, my as u32);
                        }
                    }
                    ret
                })
                .collect_into_vec(&mut tmp);
            for ((lx, ly), ret) in cand.into_iter().zip(tmp) {
                let ux = lx + dx;
                let uy = ly + dy;
                dp2[lx][ux][ly][uy].setmin(ret);
            }
        }
    }
    bar.finish();
    eprintln!("cost = {}", dp2[0][xs.len() - 1][0][ys.len() - 1].0);
    let (mut id, mut out) = all_merge(&init_canvas);
    let mut blocks = vec![BlockId(vec![id])];
    rec2(
        0,
        xs.len() - 1,
        0,
        ys.len() - 1,
        w,
        h,
        &xs,
        &ys,
        &dp,
        &dp2,
        &mut id,
        &mut blocks,
        &mut out,
        do_all_merge,
    );
    let mut canvas = init_canvas.clone();
    let mut cost = canvas.apply_all(out.clone());
    let mut expected = cost;
    for y in 0..h {
        for x in 0..w {
            for c in 0..4 {
                expected += png[y][x][c].abs_diff(canvas.bitmap[y][x][c]) as f64 * 0.005 * MUL;
            }
        }
    }
    cost += similarity(png, &canvas.bitmap);
    eprintln!("expected = {}", expected);
    eprintln!("actual = {}", cost);
    (cost, out)
}

fn rec2(
    lx: usize,
    ux: usize,
    ly: usize,
    uy: usize,
    w: usize,
    h: usize,
    xs: &Vec<usize>,
    ys: &Vec<usize>,
    dp: &Vec<Vec<Vec<Vec<(f64, (u32, u32))>>>>,
    dp2: &Vec<Vec<Vec<Vec<(f64, (u32, u32))>>>>,
    id: &mut u32,
    blocks: &mut Vec<BlockId>,
    out: &mut Vec<Move>,
    do_all_merge: bool,
) {
    match dp2[lx][ux][ly][uy].1 .0 {
        0 => {
            let x = dp2[lx][ux][ly][uy].1 .1 as usize;
            rec2(
                lx,
                x,
                ly,
                uy,
                w,
                h,
                xs,
                ys,
                dp,
                dp2,
                id,
                blocks,
                out,
                do_all_merge,
            );
            let block = blocks.pop().unwrap();
            out.push(Move::LineCut(block.clone(), 'x', xs[x] as i32));
            blocks.extend(block.cut());
            rec2(
                x,
                ux,
                ly,
                uy,
                w,
                h,
                xs,
                ys,
                dp,
                dp2,
                id,
                blocks,
                out,
                do_all_merge,
            );
            if do_all_merge || ux + 1 < xs.len() || uy + 1 < ys.len() {
                let b2 = blocks.pop().unwrap();
                let b1 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                *id += 1;
                blocks.push(BlockId(vec![*id]));
            }
        }
        1 => {
            let y = dp2[lx][ux][ly][uy].1 .1 as usize;
            rec2(
                lx,
                ux,
                ly,
                y,
                w,
                h,
                xs,
                ys,
                dp,
                dp2,
                id,
                blocks,
                out,
                do_all_merge,
            );
            let block = blocks.pop().unwrap();
            out.push(Move::LineCut(block.clone(), 'y', ys[y] as i32));
            blocks.extend(block.cut());
            rec2(
                lx,
                ux,
                y,
                uy,
                w,
                h,
                xs,
                ys,
                dp,
                dp2,
                id,
                blocks,
                out,
                do_all_merge,
            );
            if do_all_merge || ux + 1 < xs.len() || uy + 1 < ys.len() {
                let b2 = blocks.pop().unwrap();
                let b1 = blocks.pop().unwrap();
                out.push(Move::Merge(b1, b2));
                *id += 1;
                blocks.push(BlockId(vec![*id]));
            }
        }
        2 => {
            let c = dp2[lx][ux][ly][uy].1 .1;
            let color = [
                (c >> 0) as u8,
                (c >> 8) as u8,
                (c >> 16) as u8,
                (c >> 24) as u8,
            ];
            out.push(Move::Color(blocks.last().unwrap().clone(), color));
        }
        3 => {
            rec(
                xs[lx],
                xs[ux] - 1,
                ys[ly],
                ys[uy] - 1,
                w,
                h,
                dp,
                id,
                blocks,
                out,
                do_all_merge,
            );
        }
        _ => unreachable!(),
    }
}

pub fn merge_solution(init_canvas: &Canvas, s1: &Program, s2: &Program) -> Program {
    let mut out = s1.clone();
    let mut canvas = init_canvas.clone();
    canvas.apply_all(s1.clone());
    assert_eq!(canvas.blocks.len(), 1);
    let id = canvas.counter;
    for p in s2 {
        let mut p = p.clone();
        p.inc_id(id);
        out.push(p);
    }
    out
}

pub fn get_swapped_png(
    png: &Vec<Vec<[u8; 4]>>,
    program: &[Move],
    init_canvas: &Canvas,
) -> Vec<Vec<[u8; 4]>> {
    let h = png.len();
    let w = png[0].len();
    let mut dummy_png = mat![[0; 4]; h; w];
    for y in 0..h {
        for x in 0..w {
            dummy_png[y][x][0] = (y / 256) as u8;
            dummy_png[y][x][1] = y as u8;
            dummy_png[y][x][2] = (x / 256) as u8;
            dummy_png[y][x][3] = x as u8;
        }
    }
    let mut dummy_canvas = init_canvas.clone();
    dummy_canvas.bitmap = dummy_png;
    let _ = dummy_canvas.apply_all_safe(
        program
            .iter()
            .filter(|p| match p {
                Move::Color(_, _) => false,
                _ => true,
            })
            .cloned(),
    );
    let mut png2 = png.clone();
    for y in 0..h {
        for x in 0..w {
            let y2 =
                dummy_canvas.bitmap[y][x][0] as usize * 256 + dummy_canvas.bitmap[y][x][1] as usize;
            let x2 =
                dummy_canvas.bitmap[y][x][2] as usize * 256 + dummy_canvas.bitmap[y][x][3] as usize;
            png2[y2][x2] = png[y][x];
        }
    }
    png2
}

pub fn get_reversed_program(program: &Program) -> Program {
    let program = {
        let mut program = program.clone();
        let mut canvas = Canvas::new(400, 400);
        canvas.apply_all(program.clone());
        program.extend(all_merge(&canvas).1);
        program
    };
    let mut out = vec![];
    let mut rcanvas = Canvas::new(400, 400);
    for i in (0..program.len()).rev() {
        let mut canvas = Canvas::new(400, 400);
        canvas.apply_all(program[..i].iter().cloned());
        let rev = match program[i].clone() {
            Move::LineCut(b, dir, v) => {
                let b = canvas.blocks[&b].clone();
                let (b1, b2) = if dir == 'x' || dir == 'X' {
                    let b1 = rcanvas.find_block(b.0, Point(v, b.1 .1)).unwrap();
                    let b2 = rcanvas.find_block(Point(v, b.0 .1), b.1).unwrap();
                    (b1, b2)
                } else {
                    let b1 = rcanvas.find_block(b.0, Point(b.1 .0, v)).unwrap();
                    let b2 = rcanvas.find_block(Point(b.0 .0, v), b.1).unwrap();
                    (b1, b2)
                };
                Move::Merge(b1, b2)
            }
            Move::Merge(b1, b2) => {
                let b1 = canvas.blocks[&b1];
                let b2 = canvas.blocks[&b2];
                let b = rcanvas.find_block(b1.0, b2.1).unwrap();
                if b1.0 .0 == b2.0 .0 {
                    Move::LineCut(b, 'y', b2.0 .1)
                } else {
                    Move::LineCut(b, 'x', b2.0 .0)
                }
            }
            Move::Swap(b1, b2) => {
                let b1 = canvas.blocks[&b1];
                let b2 = canvas.blocks[&b2];
                let b1 = rcanvas.find_block(b1.0, b1.1).unwrap();
                let b2 = rcanvas.find_block(b2.0, b2.1).unwrap();
                Move::Swap(b1, b2)
            }
            _ => panic!("cannot reverse"),
        };
        rcanvas.apply(&rev);
        out.push(rev);
    }
    out
}
