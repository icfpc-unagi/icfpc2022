// use icfpc2022::*;

fn main() -> anyhow::Result<()> {
    let (canvas, _problem) = icfpc2022::load_problem(1);
    dbg!(canvas.counter);

    let program = icfpc2022::read_isl(std::fs::File::open(
        "/home/takiba/Dropbox/ICFPC2022/wata/out.txt",
    )?)?;

    let (canvas, _problem) = icfpc2022::load_problem(26);

    let flipped = icfpc2022::rotate::flip_program_with_initial_canvas(&program, &canvas);
    dbg!(flipped);

    let rotated = icfpc2022::rotate::rotate_program_with_initial_canvas(&program, &canvas);
    dbg!(rotated);

    // icfpc2022::rotate::

    // let image = icfpc2022::read_png("problems/1.png");
    //
    // let mut program = vec![];
    //
    // dbg!(&image[0][0]);
    // program.push(Move::Color(BlockId(vec![0]), [0, 74, 173, 255]));
    //
    // program.push(Move::PointCut(BlockId(vec![0]), 360, 40));
    // program.push(Move::Color(BlockId(vec![0, 0]), [255, 255, 255, 255]));
    //
    // let mut canvas = Canvas::new400();
    // write_apng_from_program(format!("tmp-animated.png"), &mut canvas, program, 5)?;
    // write_png(&format!("tmp.png"), canvas.bitmap)?;
    Ok(())
}
