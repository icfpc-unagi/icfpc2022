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
