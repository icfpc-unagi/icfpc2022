use std::fs::File;

use icfpc2022::{load_problem, optmerge::merge_all, write_isl};

fn main() -> anyhow::Result<()> {
    for problem_id in 26..=35 {
        let (mut canvas, _) = load_problem(problem_id);
        let moves = merge_all(&mut canvas);
        write_isl(
            File::create(format!("out/tos_merge{problem_id}.isl"))?,
            moves,
        )?;
    }
    Ok(())
}
