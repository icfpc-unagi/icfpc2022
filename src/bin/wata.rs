use icfpc2022::read_png;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let png = read_png(&input);
}
