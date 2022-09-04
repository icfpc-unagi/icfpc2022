use icfpc2022;
use icfpc2022::color::*;
use icfpc2022::read_png;
use icfpc2022::*;
//use std::collections::HashMap;

fn main() {
    let input = std::env::args().nth(1).unwrap();

    let png = read_png(&input);

    let mut blocks = vec![(BlockId(vec![0]), [0, 400, 0, 400])];
    let mut ok = vec![];

    let mut ret = vec![];
    let p = 0;

    while blocks.len() > 0 {
        let next_block = blocks.pop().unwrap();
        let ly = next_block.1[0];
        let ry = next_block.1[1];
        let lx = next_block.1[2];
        let rx = next_block.1[3];

        let dy = ry - ly;
        let dx = rx - lx;
        let bc = best_color(&png, lx, rx, ly, ry);
        eprintln!("{} {} {} {} {}", bc.1, ly, ry, lx, rx);

        if bc.1 <= 10000.0 || dy * dx <= 3 || (dy < (1 << p) * 2 && dx < (1 << p) * 2) {
            ret.push(Move::Color(next_block.clone().0, bc.0));
            ok.push(next_block.clone());
            continue;
        }

        let mut div_pos = vec![]; // (score, XorY, pos)

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
                if rx - i < i - lx {
                    score *= (rx - i) as f64;
                } else {
                    score *= (i - lx) as f64;
                }
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

                if ry - i < i - ly {
                    score *= (ry - i) as f64;
                } else {
                    score *= (i - ly) as f64;
                }
                score *= dy as f64;

                div_pos.push((score, 'Y', i));
            }
        }

        div_pos.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let div = div_pos[0];

        eprintln!("{}", div.0);
        ret.push(Move::LineCut(next_block.clone().0, div.1, div.2 as i32));
        let nblock = next_block.0.cut();
        if div.1 == 'X' {
            blocks.push((nblock[0].clone(), [ly, ry, lx, div.2]));
            blocks.push((nblock[1].clone(), [ly, ry, div.2, rx]));
        } else {
            blocks.push((nblock[0].clone(), [ly, div.2, lx, rx]));
            blocks.push((nblock[1].clone(), [div.2, ry, lx, rx]));
        }
    }

    for x in ret {
        println!("{}", x);
    }
}
