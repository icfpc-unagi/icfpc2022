use std::collections::HashMap;

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
            if let Some(mut x) = bucket.get_mut(&c) {
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
    let mut points = Vec::with_capacity((ry-ly) * (rx-lx));
    for y in ly..ry {
        for x in lx..rx {
            let p = &png[y][x];
            points.push([p[0] as f64, p[1] as f64, p[2] as f64, p[3] as f64]);
        }
    }

    let color = geometric_median_4d(&points);
    let color = color.map(|v| v as u8);

    // copy-paste...
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


fn geometric_median_4d(points: &[[f64; 4]]) -> [f64; 4] {
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
    for eps in [1e2, 1.0, 0.1, 1e-3] {  // TODO
        let dists = points.iter().map(|p| (0..4).map(|i| (p[i] - x[i]).powi(2)).sum::<f64>().sqrt()).collect::<Vec<_>>();
        let dist = dists.iter().sum::<f64>();
        // dbg!(&dists, dist);

        let x0 = x;
        x = [0.0; 4];
        let mut w_sum = 0.0;
        for p in points {
            let w = 1.0 / (eps + (0..4).map(|i| (p[i] - x0[i]).powi(2)).sum::<f64>().sqrt());
            w_sum += w;
            for i in 0..4 {
                x[i] += w * p[i];
            }
        }
        for i in 0..4 {
            x[i] /= w_sum;
        }
    }
    x
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_geometric_median_4d() {
        let points = vec![
            [0.0, 1.0, 2.0, 3.0],
            [4.0, 3.0, 2.0, 1.0],
            [3.0, 2.0, 7.0, 7.0],
        ];
        let median = geometric_median_4d(&points);
        dbg!(median);
        // assert!(false);
    }
}
