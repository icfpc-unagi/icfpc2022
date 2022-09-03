use icfpc2022::{color::best_color2, read_png, write_png};

fn main() {
    let png = read_png("problems/16.png");
    let (color, cost) = best_color2(&png, 0, 400, 0, 400);
    dbg!((color, cost));

    let png = read_png("problems/9.png");
    let mut small = vec![vec![[0; 4]; 40]; 40];
    for (y, row) in small.iter_mut().enumerate() {
        for (x, px) in row.iter_mut().enumerate() {
            *px = best_color2(&png, 10 * x, 10 * (x + 1), 10 * y, 10 * (y + 1)).0;
        }
    }
    write_png("tmp/9-small.png", small).unwrap();
}
