use std::fs::File;

use icfpc2022::{load_problem, read_isl_with_comments};

fn main() -> anyhow::Result<()> {
    let (canvas, _png) = load_problem(5);
    let (program, _comments) = read_isl_with_comments(File::open("run_id_1811.isl")?)?;
    icfpc2022::local_optimization::fix_cut_merge(program, canvas);
    Ok(())
}