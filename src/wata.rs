use crate::*;
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
    dbg!(&xys);
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
                    (dp_x[ly][uy][x].0, tmp)
                }
            };
            bar.inc(1);
            ret
        })
        .collect_into_vec(&mut dp1);
    bar.finish();

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
    let mut canvas = Canvas::new(w, h);
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
