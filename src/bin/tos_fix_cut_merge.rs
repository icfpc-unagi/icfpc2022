use std::fs::File;

use icfpc2022::{load_problem, read_isl_with_comments, write_isl_with_comments};

fn main() -> anyhow::Result<()> {
    let (canvas, png) = load_problem(5);
    let (mut program, comments) = read_isl_with_comments(File::open("run_id_1811.isl")?)?;
    let mut score = canvas.clone().apply_all_and_score(program.clone(), &png)?;
    while let Some(new_program) =
        icfpc2022::local_optimization::fix_cut_merge(program.clone(), canvas.clone())
    {
        program = new_program;
        let new_score = canvas.clone().apply_all_and_score(program.clone(), &png)?;
        eprintln!("***** {score} -> {new_score}");
        assert!(new_score <= score);
        score = new_score;
    }
    write_isl_with_comments(File::create("out/edited_1811.isl")?, program, comments)?;
    Ok(())
}
