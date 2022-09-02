use icfpc2022;



fn main() {
    let input = std::env::args().nth(1).unwrap();
    let png = icfpc2022::read_png(&input);

    dbg!(icfpc2022::color::best_color(&png, 0, 0, 10, 10));
}
