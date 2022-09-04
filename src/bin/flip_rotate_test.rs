// use icfpc2022::*;

fn main() -> anyhow::Result<()> {
    let program = icfpc2022::read_isl(std::fs::File::open(
        "/home/takiba/Dropbox/ICFPC2022/wata/out.txt",
    )?)?;

    let (canvas, _problem) = icfpc2022::load_problem(26);

    let flipped = icfpc2022::rotate::flip_program_with_initial_canvas(&program, &canvas);
    icfpc2022::write_isl(std::fs::File::create("flipped.txt")?, flipped)?;

    let rotated = icfpc2022::rotate::rotate_program_with_initial_canvas(&program, &canvas);
    icfpc2022::write_isl(std::fs::File::create("rotated.txt")?, rotated)?;

    Ok(())
}
