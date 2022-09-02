use icfpc2022;

fn main() {
    //let input = std::env::args().nth(1).unwrap();
    //let png = icfpc2022::read_png(&input);

    // let program = vec![icfpc2022::Move::Color(icfpc::BlockId(vec![0]))];
    let program = icfpc2022::read_isl(std::io::stdin()).unwrap();
    dbg!(&program);

    let program2 = icfpc2022::rotate::rotate_program(&program);
    dbg!(&program2);

    let program2 = icfpc2022::rotate::flip_program(&program);
    dbg!(&program2);
}
