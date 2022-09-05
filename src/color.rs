use std::collections::HashMap;

use crate::{pixel_distance, Color};

#[deprecated]
pub fn best_color(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> ([u8; 4], f64) {
    let mut s: [f64; 4] = [0.0; 4];

    for y in ly..ry {
        for x in lx..rx {
            for c in 0..4 {
                s[c] += png[y][x][c] as f64;
            }
        }
    }

    let color = s.map(|x| (x / ((rx - lx) as f64) / ((ry - ly) as f64)) as u8);

    let mut cost = 0.0;
    for y in ly..ry {
        for x in lx..rx {
            let mut t = 0.0;
            for c in 0..4 {
                let d = (color[c] as f64) - (png[y][x][c] as f64);
                t += d * d
            }
            cost += t.sqrt();
        }
    }

    return (color, cost);
}

fn cost(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
    color: [u8; 4],
) -> f64 {
    let mut cost = 0.0;
    for y in ly..ry {
        for x in lx..rx {
            let mut t = 0.0;
            for c in 0..4 {
                let d = (color[c] as f64) - (png[y][x][c] as f64);
                t += d * d
            }
            cost += t.sqrt();
        }
    }
    cost
}

pub fn median_color(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> ([u8; 4], f64) {
    let area = (rx - lx) * (ry - ly);
    let color;
    if area <= 100 {
        color = median_color_by_sort(png, lx, rx, ly, ry);
    } else {
        color = median_color_by_bucketing(png, lx, rx, ly, ry);
    }
    (color, cost(png, lx, rx, ly, ry, color))
}

fn median_bucket(b: [usize; 256], n: usize) -> u8 {
    assert!(n >= 1);
    let k = (n - 1) / 2;
    let mut s = 0;
    for i in 0..256 {
        s += b[i];
        if s > k {
            return i as u8;
        }
    }
    panic!();
}

pub fn median_color_by_bucketing(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> [u8; 4] {
    let mut buckets = [[0_usize; 256]; 4];
    for y in ly..ry {
        for x in lx..rx {
            for c in 0..4 {
                buckets[c][png[y][x][c] as usize] += 1;
            }
        }
    }

    let n = (rx - lx) * (ry - ly);
    buckets.map(|b| median_bucket(b, n))
}

fn median_array(mut a: Vec<u8>) -> u8 {
    a.sort();
    let n = a.len();
    assert!(n >= 1);
    // TODO: そのうち暇な時、偶数の時にあれする
    return a[(n - 1) / 2];
}

pub fn median_color_by_sort(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> [u8; 4] {
    let mut points = [
        Vec::with_capacity((ry - ly) * (rx - lx)),
        Vec::with_capacity((ry - ly) * (rx - lx)),
        Vec::with_capacity((ry - ly) * (rx - lx)),
        Vec::with_capacity((ry - ly) * (rx - lx)),
    ];
    for y in ly..ry {
        for x in lx..rx {
            for c in 0..4 {
                points[c].push(png[y][x][c]);
            }
        }
    }
    points.map(|p| median_array(p))
}

pub fn mode_color(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> ([u8; 4], f64) {
    let mut bucket = HashMap::new();

    for y in ly..ry {
        for x in lx..rx {
            let c = png[y][x];
            if let Some(x) = bucket.get_mut(&c) {
                *x += 1;
            } else {
                bucket.insert(c, 1u32);
            }
        }
    }

    let color = *bucket.iter().max_by_key(|x| x.1).unwrap().0;

    let mut cost = 0.0;
    for y in ly..ry {
        for x in lx..rx {
            let mut t = 0.0;
            for c in 0..4 {
                let d = (color[c] as f64) - (png[y][x][c] as f64);
                t += d * d
            }
            cost += t.sqrt();
        }
    }

    return (color, cost);
}

pub fn best_color2(
    png: &Vec<Vec<[u8; 4]>>,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> ([u8; 4], f64) {
    let mut points = Vec::with_capacity((ry - ly) * (rx - lx));
    let mut u8_points = Vec::with_capacity((ry - ly) * (rx - lx));
    for y in ly..ry {
        for x in lx..rx {
            let p = png[y][x];
            u8_points.push(p);
            points.push([p[0] as f64, p[1] as f64, p[2] as f64, p[3] as f64]);
        }
    }
    let u8_points = u8_points;

    let color = geometric_median_4d(&points);

    round_to_optimal_u8_color(&u8_points, &color)
}

pub fn round_to_optimal_u8_color(u8_points: &Vec<[u8; 4]>, f64_color: &[f64; 4]) -> ([u8; 4], f64) {
    let color = f64_color.map(|v| v.floor().clamp(0.0, 254.0) as u8);

    let mut best = 1e99;
    let mut best_color = [0; 4];
    // local search
    for flags in 0..(1 << 4) {
        let color = [
            color[0] + ((flags >> 0) & 1),
            color[1] + ((flags >> 1) & 1),
            color[2] + ((flags >> 2) & 1),
            color[3] + ((flags >> 3) & 1),
        ];
        let cost = u8_points
            .iter()
            .map(|p| pixel_distance(p, &color))
            .sum::<f64>();
        if cost < best {
            best = cost;
            best_color = color;
        }
    }
    return (best_color, best);
}

#[allow(unused)]
fn dbg_point(p: [f64; 4]) {
    eprintln!("{:10.4} {:10.4} {:10.4} {:10.4} ", p[0], p[1], p[2], p[3])
}

pub fn geometric_median_4d(points: &[[f64; 4]]) -> [f64; 4] {
    let n = points.len();
    let mut x = [0.0; 4];
    for p in points {
        for i in 0..4 {
            x[i] += p[i];
        }
    }
    for i in 0..4 {
        x[i] /= n as f64;
    }

    // let mut momentum = [0.0; 5];
    // let gamma = 0.9;

    // TODO: fix eps
    // 誤差0.5程度におさえられるくらいに調整したい
    let mut eps = 1.0;
    for iter in 0..100 {
        // dbg_point(x);
        // let min_step_size = 10.0 * 0.01_f64.powf(iter as f64 / 29.0);
        // let eps = if iter < 10 {
        //     1.0
        // } else if iter < 20 {
        //     0.1
        // } else {
        //     0.01
        // };
        // let dists = points
        //     .iter()
        //     .map(|p| (0..4).map(|i| (p[i] - x[i]).powi(2)).sum::<f64>().sqrt())
        //     .collect::<Vec<_>>();
        // let dist = dists.iter().sum::<f64>();
        // dbg!(&dists, dist);

        // dbg!(&x);

        // let x0 = x;
        // x = [0.0; 4];
        let mut grad = [0.0; 4];
        let mut w_sum = 0.0;
        for p in points {
            let diff = [p[0] - x[0], p[1] - x[1], p[2] - x[2], p[3] - x[3]];
            let dist = diff.iter().map(|d| d.powi(2)).sum::<f64>().sqrt();
            // (huber loss)' == clip
            let w = 1.0 / f64::max(eps, dist);
            w_sum += w;
            for i in 0..4 {
                grad[i] += w * diff[i];
            }
        }

        // Ostresh (1978): $\lambda \in [1, 2]$ can be used here (?)
        let lr = 1.8 / w_sum;
        // eprintln!("iter = {}, lr = {}", iter, 1.0 / w_sum);

        for i in 0..4 {
            grad[i] *= lr;
        }
        let step_size = grad.iter().map(|g| g.powi(2)).sum::<f64>().sqrt();
        if step_size < eps {
            if eps < 1e-2 {
                // u8に戻すときに16通り調べるのでearly returnの目標はそのオーダー
                // eprintln!("early return: {iter}");
                break;
            }
            eps *= 0.5;
            // dbg!(eps);
        }
        // dbg!(step_size, min_step_size);
        // if step_size < min_step_size {
        //     let fix = min_step_size / (1e-6 + step_size);
        //     for i in 0..4{
        //     grad[i] *= fix;}
        // }
        for i in 0..4 {
            x[i] += grad[i];
        }

        // for i in 0..4 {
        //     momentum[i] += (1.0 - gamma) * (grad[i] - momentum[i])
        // }
        // momentum[4] += (1.0 - gamma) * (1.0 - momentum[4]);
        // for i in 0..4 {
        //     x[i] += lr * (momentum[i] / momentum[4]);
        // }
    }
    x
}

const Z: usize = 256;

struct MedianSelector {
    /// [0, y) * [0, x) * [0, z) の和を保持
    sum_cnt: Vec<Vec<[u32; Z + 1]>>,
    sum_val: Vec<Vec<[u32; Z + 1]>>,
}

impl MedianSelector {
    fn new(image: &Vec<Vec<Color>>, channel: usize) -> Self {
        let h = image.len();
        let w = image[0].len();

        let mut sum_count = vec![vec![[0; Z + 1]; w + 1]; h + 1];
        let mut sum_value = vec![vec![[0; Z + 1]; w + 1]; h + 1];
        for y in 0..h {
            for x in 0..w {
                let z = image[y][x][channel] as usize;
                sum_count[y + 1][x + 1][z + 1] += 1;
                sum_value[y + 1][x + 1][z + 1] += z as u32;
            }
        }

        for y in 0..=h {
            for x in 0..=w {
                for z in 1..=Z {
                    sum_count[y][x][z] += sum_count[y][x][z - 1];
                    sum_value[y][x][z] += sum_value[y][x][z - 1];
                }
            }
        }
        for y in 0..=h {
            for z in 0..=Z {
                for x in 1..=w {
                    sum_count[y][x][z] += sum_count[y][x - 1][z];
                    sum_value[y][x][z] += sum_value[y][x - 1][z];
                }
            }
        }
        for x in 0..=w {
            for z in 0..=Z {
                for y in 1..=h {
                    sum_count[y][x][z] += sum_count[y - 1][x][z];
                    sum_value[y][x][z] += sum_value[y - 1][x][z];
                }
            }
        }

        Self {
            sum_cnt: sum_count,
            sum_val: sum_value,
        }
    }

    /// [lx, rx) * [ly, ry) * [0, z)
    fn sum_count_rectangle(&self, lx: usize, rx: usize, ly: usize, ry: usize, z: usize) -> u32 {
        self.sum_cnt[ry][rx][z] + self.sum_cnt[ly][lx][z]
            - self.sum_cnt[ry][lx][z]
            - self.sum_cnt[ly][rx][z]
    }

    /// [lx, rx) * [ly, ry) * [0, z)
    fn sum_value_rectangle(&self, lx: usize, rx: usize, ly: usize, ry: usize, z: usize) -> u32 {
        self.sum_val[ry][rx][z] + self.sum_val[ly][lx][z]
            - self.sum_val[ry][lx][z]
            - self.sum_val[ly][rx][z]
    }

    /// [lx, rx) * [ly, ry) のk番目
    fn kth_rectangle(&self, lx: usize, rx: usize, ly: usize, ry: usize, k: usize) -> u8 {
        let mut z = 0;
        for lev in (0..8).rev() {
            let b = 1 << lev;
            let s = self.sum_count_rectangle(lx, rx, ly, ry, z | b) as usize;
            if s <= k {
                z |= b;
            }
        }

        z as u8
    }

    fn l1_rectangle(&self, lx: usize, rx: usize, ly: usize, ry: usize, z: usize) -> u32 {
        let cnt_below = self.sum_count_rectangle(lx, rx, ly, ry, z);
        let cnt_above = self.sum_count_rectangle(lx, rx, ly, ry, 256) - cnt_below;
        let sum_below = self.sum_value_rectangle(lx, rx, ly, ry, z);
        let sum_above = self.sum_value_rectangle(lx, rx, ly, ry, 256) - sum_below;
        // dbg!(
        //     cnt_below,
        //     cnt_above,
        //     sum_below,
        //     sum_above,
        //     cnt_below * (z as u32) - sum_below,
        //     (sum_above - cnt_above * (z as u32))
        // );
        (cnt_below * (z as u32) - sum_below) + (sum_above - cnt_above * (z as u32))
    }
}

pub struct MedianColorSelector {
    selectors: [MedianSelector; 4],
}

impl MedianColorSelector {
    pub fn new(image: &Vec<Vec<Color>>) -> Self {
        Self {
            selectors: [
                MedianSelector::new(&image, 0),
                MedianSelector::new(&image, 1),
                MedianSelector::new(&image, 2),
                MedianSelector::new(&image, 3),
            ],
        }
    }

    pub fn query(&self, lx: usize, rx: usize, ly: usize, ry: usize) -> (Color, u32) {
        let area = (rx - lx) * (ry - ly);
        assert!(area >= 1);
        let k = (area - 1) / 2;

        let mut color = [0; 4];
        let mut cost = 0;
        for (i, selector) in self.selectors.iter().enumerate() {
            color[i] = selector.kth_rectangle(lx, rx, ly, ry, k);
            cost += selector.l1_rectangle(lx, rx, ly, ry, color[i] as usize);
        }

        (color, cost)
    }
}

pub fn l1_naive(
    image: &Vec<Vec<Color>>,
    color: &Color,
    lx: usize,
    rx: usize,
    ly: usize,
    ry: usize,
) -> u32 {
    let mut sum = 0;
    for y in ly..ry {
        for x in lx..rx {
            for c in 0..4 {
                sum += ((image[y][x][c] as i64) - (color[c] as i64)).abs()
            }
        }
    }
    assert!(sum >= 0);
    sum as u32
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn point_sub(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]]
    }

    fn point_inner(a: [f64; 4], b: [f64; 4]) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
    }

    fn point_cos(a: [f64; 4], b: [f64; 4]) -> f64 {
        point_inner(a, b) / (point_inner(a, a) * point_inner(b, b)).sqrt()
    }

    fn point_from_u8(a: [u8; 4]) -> [f64; 4] {
        [a[0] as f64, a[1] as f64, a[2] as f64, a[3] as f64]
    }

    #[test]
    fn test_geometric_median_4d() {
        let a = [0.0, 10.0, 20.0, 30.0];
        let b = [40.0, 30.0, 20.0, 10.0];
        let c = [30.0, 20.0, 70.0, 70.0];
        let points = vec![a, b, c];
        let median = geometric_median_4d(&points);
        let diffs = points
            .iter()
            .copied()
            .map(|p| point_sub(p, median))
            .collect::<Vec<_>>();
        dbg!(median);
        for (i, j) in [(0, 1), (0, 2), (1, 2)] {
            let x = diffs[i];
            let y = diffs[j];
            // assert angle is about 120 deg
            let c = point_cos(x, y);
            dbg!(c);
            assert!((c + 0.5).abs() < 0.01)
        }

        let points = vec![c, c, a, b, c, a, c];
        let median = geometric_median_4d(&points);
        // dbg!(median);
        assert_eq!(median.map(|v| v.round()), c);

        let mut points = vec![];
        for r in 10..20 {
            for g in 30..35 {
                for b in 40..43 {
                    points.push(point_from_u8([r, g, b, 0]));
                }
            }
        }
        for _ in 0..149 {
            points.push(point_from_u8([255, 255, 255, 255]));
        }
        let median = geometric_median_4d(&points);
        // 収束してなさそう・・・。
        dbg!(median);
        // assert!(false);
    }

    #[test]
    fn test_best_color2() {
        let a = [0, 10, 20, 30];
        let b = [40, 30, 20, 10];
        let c = [30, 20, 70, 70];
        let points = vec![a, b, c];
        let png = vec![points.clone()];
        let (point, cost) = best_color2(&png, 0, points.len(), 0, 1);
        dbg!(point, cost);
        // 今の出力を答えにしただけなので違ったら直して
        assert_eq!(point, [19, 19, 30, 31]);

        let png = vec![vec![b, a, b, a], vec![a, a, a, c]];
        let (point, _cost) = best_color2(&png, 0, 4, 0, 2);
        assert_eq!(point, a); // mode
    }

    fn minmax(a: usize, b: usize) -> (usize, usize) {
        (a.min(b), a.max(b))
    }

    #[test]
    fn test_median_color() {
        let image = crate::read_png("problems/16.png");
        let h = image.len();
        let w = image[0].len();
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([13; 32]);

        for _ in 0..100 {
            let (lx, rx) = minmax(rng.gen::<usize>() % w, rng.gen::<usize>() % w);
            let rx = rx + 1;

            let (ly, ry) = minmax(rng.gen::<usize>() % h, rng.gen::<usize>() % h);
            let ry = ry + 1;

            let c1 = median_color_by_bucketing(&image, lx, rx, ly, ry);
            let c2 = median_color_by_sort(&image, lx, rx, ly, ry);
            assert_eq!(c1, c2);
        }
    }

    #[test]
    fn test_median_color_selector() {
        let mut image = crate::read_png("problems/16.png");
        // くそでか状態でやるとおそすぎてやばいので小さくする
        image.truncate(30);
        image.iter_mut().for_each(|row| row.truncate(40));

        let h = image.len();
        let w = image[0].len();

        let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([13; 32]);
        let selector = MedianColorSelector::new(&image);

        check_median_selector(&image, &selector, 0, w, 0, h);
        check_median_selector(&image, &selector, 0, 1, 0, 1);
        check_median_selector(&image, &selector, w - 1, w, h - 1, h);

        for _ in 0..100 {
            let (lx, rx) = minmax(rng.gen::<usize>() % w, rng.gen::<usize>() % w);
            let rx = rx + 1;

            let (ly, ry) = minmax(rng.gen::<usize>() % h, rng.gen::<usize>() % h);
            let ry = ry + 1;

            check_median_selector(&image, &selector, lx, rx, ly, ry);
        }
    }

    fn check_median_selector(
        image: &Vec<Vec<[u8; 4]>>,
        selector: &MedianColorSelector,
        lx: usize,
        rx: usize,
        ly: usize,
        ry: usize,
    ) {
        let (c1, _) = median_color(&image, lx, rx, ly, ry);
        let l1 = l1_naive(&image, &c1, lx, rx, ly, ry);

        let (c2, l2) = selector.query(lx, rx, ly, ry);
        assert_eq!(c1, c2);
        assert_eq!(l1, l2);
    }
}
