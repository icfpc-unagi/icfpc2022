fn main() {
    let image = icfpc2022::read_png("problems/16.png");

    let lx = 100;
    let rx = 110;
    let ly = 100;
    let ry = 110;

    let c1 = icfpc2022::color::median_color_by_bucketing(&image, lx, rx, ly, ry);
    let c2 = icfpc2022::color::median_color_by_sort(&image, lx, rx, ly, ry);

    let selector = icfpc2022::color::MedianColorSelector::new(&image);
    let c3 = selector.query(lx, rx, ly, ry);

    let l1 = icfpc2022::color::l1_naive(&image, &c3.0, lx, rx, ly, ry);
    dbg!(c1, c2, c3, l1);
}
