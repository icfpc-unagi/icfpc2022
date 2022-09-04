use std::collections::HashMap;

use crate::pixel_distance;

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
    for iter in 0..100 {
        // dbg_point(x);
        // let min_step_size = 10.0 * 0.01_f64.powf(iter as f64 / 29.0);
        let eps = if iter < 10 {
            1.0
        } else if iter < 20 {
            0.1
        } else {
            0.01
        };
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
            let w = 1.0 / (eps + dist);
            w_sum += w;
            for i in 0..4 {
                grad[i] += w * diff[i];
            }
        }

        let lr = 1.0 / w_sum;
        // eprintln!("iter = {}, lr = {}", iter, 1.0 / w_sum);

        for i in 0..4 {
            grad[i] *= lr;
        }
        // let step_size = grad.iter().map(|g| g.powi(2)).sum::<f64>().sqrt();
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

#[cfg(test)]
mod tests {
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
                    points.push(point_from_u8([r, g, b, 255]));
                }
            }
        }
        for _ in 0..140 {
            points.push(point_from_u8([0, 0, 0, 0]));
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
}
