use std::fs::File;

use icfpc2022::{load_problem, optmerge::merge_all, write_isl};

fn main() -> anyhow::Result<()> {
    let (mut canvas, _) = load_problem(26);
    let moves = merge_all(&mut canvas);
    write_isl(File::create("out/tos_merge26.isl")?, moves)?;
    Ok(())
}
