fn main() {
    let image = icfpc2022::read_png("problems/1.png");

    let c1 = icfpc2022::color::median_color_by_bucketing(&image, 100, 300, 100, 300);
    let c2 = icfpc2022::color::median_color_by_sort(&image, 100, 300, 100, 300);
    dbg!(c1, c2);
}
