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
            let mut out = vec![];
            let mut y = h;
            while y > 0 {
                out.push((y, dp[y].2));
                y = dp[y].1;
            }
            out.reverse();
            (lx, ux, (dp[h].0, out))
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
        // 潜伏(14+2000*hiding)
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
