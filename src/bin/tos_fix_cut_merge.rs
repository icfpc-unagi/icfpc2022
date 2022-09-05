use std::fs::File;

use icfpc2022::{load_problem, read_isl_with_comments, write_isl_with_comments};

fn main() -> anyhow::Result<()> {
    // let (id, filename) = (5, "run_id_1811.isl");
    let (id, filename) = (27, "/Users/tos/Dropbox/ICFPC2022/runs/3650.isl");

    // submissions::read_submission_program_problem(submission.id)?;
    let (canvas, png) = load_problem(id);
    let (mut program, comments) = read_isl_with_comments(File::open(filename)?)?;
    let mut score = canvas.clone().apply_all_and_score(program.clone(), &png)?;
    while let Some(new_program) =
        icfpc2022::local_optimization::fix_cut_merge(program.clone(), canvas.clone())
    {
        program = new_program;
        // println!("hoge");
        // break;
        let new_score = canvas.clone().apply_all_and_score(program.clone(), &png)?;
        eprintln!("***** {score} -> {new_score}");
        assert!(new_score <= score);
        score = new_score;
    }
    write_isl_with_comments(File::create("out/optfailed_3650.isl")?, program, comments)?;
    Ok(())
}
