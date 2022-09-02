use icfpc2022::read_png;
//use std::collections::HashMap;

struct Global {
    max_color: usize,
}

struct Carry<'a> {
    memo: &'a mut Vec<Vec<Vec<Vec<Vec<f64>>>>>, // メモ化した答え
    memo_type: &'a mut Vec<Vec<Vec<Vec<Vec<usize>>>>>, // メモしたやること
    memo_c: &'a mut Vec<Vec<Vec<Vec<Vec<usize>>>>>, // メモしたCに対してやること
    memo_y: &'a mut Vec<Vec<Vec<Vec<Vec<usize>>>>>, // メモしたYに対してやること
    memo_x: &'a mut Vec<Vec<Vec<Vec<Vec<usize>>>>>, // メモしたXに対してやること
    pos_y: &'a mut Vec<usize>,                  // Y座標
    pos_x: &'a mut Vec<usize>,                  // X座標
    colors: &'a mut Vec<[u8; 4]>,               // 候補色
    pixel_error: &'a mut Vec<Vec<Vec<f64>>>,    // PixelDiffirenceの二次元累積和
}

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let png = read_png(&input);
    let campus_size = 400 as usize;

    let mut colors: Vec<[u8; 4]> = vec![[255, 255, 255, 255], [0, 0, 0, 255], [0, 74, 173, 255]];
    /*
    let mut colors: Vec<[u8; 4]> = Vec::new();
    let mut color_hash: HashMap<[u8; 4], usize> = HashMap::new();

    *color_hash.entry([255,255,255,255]).or_insert(0);

    for y in 0..campus_size {
        for x in 0..campus_size {
            *color_hash.entry(png[y][x]).or_insert(0) += 1;
        }
    }

    for a in color_hash.keys() {
        colors.push(a.clone());
    }
    */

    let mut pos_x: Vec<usize> = vec![0, 40, 80, 120, 160, 200, 240, 280, 320, 360, 400];
    let mut pos_y: Vec<usize> = vec![0, 40, 80, 120, 160, 200, 240, 280, 320, 360, 400];
    let gl = &mut Global {
        max_color: colors.len(),
    };

    eprintln!("Color: {}", colors.len());

    let ysize = pos_y.len();
    let xsize = pos_x.len();
    let csize = colors.len();

    let mut memo =
        vec![vec![vec![vec![vec![0.0; csize + 1]; xsize + 1]; ysize + 1]; xsize + 1]; ysize + 1];
    let mut memo_c =
        vec![
            vec![vec![vec![vec![0 as usize; csize + 1]; xsize + 1]; ysize + 1]; xsize + 1];
            ysize + 1
        ];
    let mut memo_y =
        vec![
            vec![vec![vec![vec![0 as usize; csize + 1]; xsize + 1]; ysize + 1]; xsize + 1];
            ysize + 1
        ];
    let mut memo_x =
        vec![
            vec![vec![vec![vec![0 as usize; csize + 1]; xsize + 1]; ysize + 1]; xsize + 1];
            ysize + 1
        ];
    let mut memo_type =
        vec![
            vec![vec![vec![vec![0 as usize; csize + 1]; xsize + 1]; ysize + 1]; xsize + 1];
            ysize + 1
        ];

    let mut pixel_error = vec![vec![vec![0.0; xsize + 1]; ysize + 1]; csize];
    for c in 0..csize {
        for y in 0..campus_size {
            let mut yp = 0;
            while y >= pos_y[yp] {
                yp += 1;
            }
            for x in 0..campus_size {
                let mut xp = 0;
                while x >= pos_x[xp] {
                    xp += 1;
                }
                pixel_error[c][yp][xp] += get_diff(colors[c], png[y][x]);
            }
        }
    }
    for c in 0..csize {
        for y in 1..ysize + 1 {
            for x in 1..xsize + 1 {
                pixel_error[c][y][x] += pixel_error[c][y - 1][x];
                pixel_error[c][y][x] += pixel_error[c][y][x - 1];
                pixel_error[c][y][x] -= pixel_error[c][y - 1][x - 1];
            }
        }
    }

    let mut car = Carry {
        memo: &mut memo,
        memo_type: &mut memo_type,
        memo_c: &mut memo_c,
        memo_y: &mut memo_y,
        memo_x: &mut memo_x,
        pos_y: &mut pos_y,
        pos_x: &mut pos_x,
        colors: &mut colors,
        pixel_error: &mut &mut pixel_error,
    };

    println!(
        "# Score : {}",
        dfs(0, 0, ysize - 1, xsize - 1, 0, &mut car, gl)
    );
    let mut next_id: usize = 0;
    dfs2(
        0,
        0,
        ysize - 1,
        xsize - 1,
        0,
        "0",
        &mut next_id,
        &mut car,
        gl,
    );
}

fn get_diff(a: [u8; 4], b: [u8; 4]) -> f64 {
    f64::sqrt(p2(a[0], b[0]) + p2(a[1], b[1]) + p2(a[2], b[2]) + p2(a[3], b[3])) * 0.005
}

fn p2(a: u8, b: u8) -> f64 {
    (a as f64 - b as f64) * (a as f64 - b as f64)
}

fn dfs2(
    ly: usize,    // 左(圧縮済み)
    lx: usize,    // 上
    ry: usize,    // 右
    rx: usize,    // 下
    color: usize, // 色
    sid: &str,
    next_id: &mut usize,
    car: &mut Carry,
    gl: &mut Global,
) {
    let cmemo = car.memo_c[ly][lx][ry][rx][color];

    if car.memo_type[ly][lx][ry][rx][color] == 1 {
        return;
    } else if car.memo_type[ly][lx][ry][rx][color] == 2 {
        println!("color [{}] [{}]", sid, color_string(car.colors[cmemo]));
        println!(
            "#pos {} {} {} {}",
            car.pos_y[ly], car.pos_x[lx], car.pos_y[ry], car.pos_x[rx]
        );
    } else if car.memo_type[ly][lx][ry][rx][color] == 3 {
        if color != car.memo_c[ly][lx][ry][rx][color] {
            println!("color [{}] [{}]", sid, color_string(car.colors[cmemo]));
            println!(
                "#pos {} {} {} {}",
                car.pos_y[ly], car.pos_x[lx], car.pos_y[ry], car.pos_x[rx]
            );
        }
        let p = car.memo_y[ly][lx][ry][rx][color].clone();
        println!("cut [{}] [{}] [{}]", sid, "Y", car.pos_y[p]);

        dfs2(
            ly,
            lx,
            p,
            rx,
            car.memo_c[ly][lx][ry][rx][color],
            &(sid.to_owned() + ".0"),
            next_id,
            car,
            gl,
        );
        dfs2(
            p,
            lx,
            ry,
            rx,
            car.memo_c[ly][lx][ry][rx][color],
            &(sid.to_owned() + ".1"),
            next_id,
            car,
            gl,
        );
    } else if car.memo_type[ly][lx][ry][rx][color] == 4 {
        if color != car.memo_c[ly][lx][ry][rx][color] {
            println!("color [{}] [{}]", sid, color_string(car.colors[cmemo]));
            println!(
                "#pos {} {} {} {}",
                car.pos_y[ly], car.pos_x[lx], car.pos_y[ry], car.pos_x[rx]
            );
        }
        let p = car.memo_x[ly][lx][ry][rx][color].clone();
        println!("cut [{}] [{}] [{}]", sid, "X", car.pos_x[p]);

        dfs2(
            ly,
            lx,
            ry,
            p,
            car.memo_c[ly][lx][ry][rx][color],
            &(sid.to_owned() + ".0"),
            next_id,
            car,
            gl,
        );
        dfs2(
            ly,
            p,
            ry,
            rx,
            car.memo_c[ly][lx][ry][rx][color],
            &(sid.to_owned() + ".1"),
            next_id,
            car,
            gl,
        );
    }
}

fn color_string(color: [u8; 4]) -> String {
    format!("{},{},{},{}", color[0], color[1], color[2], color[3]).to_string()
}

fn dfs(
    ly: usize,    // 左(圧縮済み)
    lx: usize,    // 上
    ry: usize,    // 右
    rx: usize,    // 下
    color: usize, // 色
    car: &mut Carry,
    gl: &mut Global,
) -> f64 {
    if car.memo_type[ly][lx][ry][rx][color] != 0 {
        return car.memo[ly][lx][ry][rx][color];
    }

    let block_size = get_1d_sum(ly, ry, car.pos_y) * get_1d_sum(lx, rx, car.pos_x);
    let mul_penalty = 400.0 * 400.0 / (block_size as f64);

    //そのままで終わりパターン
    car.memo[ly][lx][ry][rx][color] = get_2d_sum(ly, lx, ry, rx, &mut car.pixel_error[color]);
    car.memo_type[ly][lx][ry][rx][color] = 1;

    /*
    println!(
        "a1: {} {} {} {} {} {}",
        ly, lx, ry, rx, color, car.memo[ly][lx][ry][rx][color]
    );
    */

    //色塗って終わりパターン

    for c in 0..gl.max_color {
        let tmp = {
            if c == color {
                get_2d_sum(ly, lx, ry, rx, &mut car.pixel_error[c])
            } else {
                get_2d_sum(ly, lx, ry, rx, &mut car.pixel_error[c]) + 5.0 * mul_penalty
            }
        };

        if car.memo[ly][lx][ry][rx][color] >= tmp {
            car.memo[ly][lx][ry][rx][color] = tmp;
            car.memo_type[ly][lx][ry][rx][color] = 2;
            car.memo_c[ly][lx][ry][rx][color] = c;
        }
    }

    /*
    println!(
        "a2: {} {} {} {} {}",
        ly, lx, ry, rx, car.memo[ly][lx][ry][rx][color]
    );
    */

    //LineCutYパターン
    for c in 0..gl.max_color {
        let tmp = {
            if c == color {
                7.0 * mul_penalty
            } else {
                12.0 * mul_penalty
            }
        };
        for p in ly + 1..ry {
            let tmp2 = tmp + dfs(ly, lx, p, rx, c, car, gl) + dfs(p, lx, ry, rx, c, car, gl);

            if car.memo[ly][lx][ry][rx][color] >= tmp2 {
                car.memo[ly][lx][ry][rx][color] = tmp2;
                car.memo_type[ly][lx][ry][rx][color] = 3;
                car.memo_c[ly][lx][ry][rx][color] = c;
                car.memo_y[ly][lx][ry][rx][color] = p;
            }
        }
    }

    /*
    println!(
        "a3: {} {} {} {} {}",
        ly, lx, ry, rx, car.memo[ly][lx][ry][rx][color]
    );
    */

    //LineCutよこパターン
    for c in 0..gl.max_color {
        let tmp = {
            if c == color {
                7.0 * mul_penalty
            } else {
                12.0 * mul_penalty
            }
        };
        for p in lx + 1..rx {
            let tmp2 = tmp + dfs(ly, lx, ry, p, c, car, gl) + dfs(ly, p, ry, rx, c, car, gl);

            if car.memo[ly][lx][ry][rx][color] >= tmp2 {
                car.memo[ly][lx][ry][rx][color] = tmp2;
                car.memo_type[ly][lx][ry][rx][color] = 4;
                car.memo_c[ly][lx][ry][rx][color] = c;
                car.memo_x[ly][lx][ry][rx][color] = p;
            }
        }
    }
    /*
    println!(
        "a4: {} {} {} {} {}",
        ly, lx, ry, rx, car.memo[ly][lx][ry][rx][color]
    );
    */

    //TODO:PointCutパターン
    for c in 0..gl.max_color {
        let tmp = {
            if c == color {
                1.0 * mul_penalty
            } else {
                1.0 * mul_penalty
            }
        };
        for p in ly + 1..ry {
            for q in lx + 1..rx {
                let tmp2 = tmp
                    + dfs(ly, lx, p, q, c, car, gl)
                    + dfs(ly, q, p, rx, c, car, gl)
                    + dfs(p, lx, ry, q, c, car, gl)
                    + dfs(p, q, ry, rx, c, car, gl);

                if car.memo[ly][lx][ry][rx][color] >= tmp2 {
                    car.memo[ly][lx][ry][rx][color] = tmp2;
                    car.memo_type[ly][lx][ry][rx][color] = 5;
                    car.memo_c[ly][lx][ry][rx][color] = c;
                    car.memo_y[ly][lx][ry][rx][color] = p;
                    car.memo_x[ly][lx][ry][rx][color] = q;
                }
            }
        }
    }

    //TODO:PointCut＋Mergeで3分割パターン

    car.memo[ly][lx][ry][rx][color]
}

fn get_2d_sum(
    ly: usize, // 左(圧縮済み)
    lx: usize, // 上
    ry: usize, // 右
    rx: usize, // 下
    sum: &mut Vec<Vec<f64>>,
) -> f64 {
    sum[ry][rx] - sum[ly][rx] - sum[ry][lx] + sum[ly][lx]
}

fn get_1d_sum(
    lx: usize, // 左(圧縮済み)
    rx: usize, // 右
    sum: &mut Vec<usize>,
) -> usize {
    sum[rx] - sum[lx]
}
