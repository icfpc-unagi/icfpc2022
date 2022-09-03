use std::fs::File;

use icfpc2022::{read_isl, read_png, write_isl, Canvas};

fn main1(problem_id: u32, submission_id: u32) -> anyhow::Result<()> {
    let mut f = File::open(format!("submissions/{submission_id}.isl"))?;

    let mut program = read_isl(&mut f)?;
    let mut changed = false;
    while !program.last().unwrap().may_change_bitmap() {
        changed = true;
        program.pop();
    }
    if !changed {
        // OK
        return Ok(());
    }

    let answer = read_png(&format!("problems/{problem_id}.png"));
    let score = Canvas::new400().apply_all_and_score(program.clone(), &answer)?;
    let mut f_out = File::create(format!("out/opt_{}_{:06.0}", problem_id, score))?;
    write_isl(&mut f_out, program)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, icfpc2022::Submission>::new();

    // from local_optimize_all.rs
    for json_path in glob::glob("submissions/*.json")? {
        let json_path = json_path?;
        let submission: icfpc2022::Submission =
            serde_json::from_reader(std::fs::File::open(json_path)?)?;

        best_submissions
            .entry(submission.problem_id)
            .and_modify(|s| {
                if s.cost > submission.cost {
                    *s = submission.clone()
                }
            })
            .or_insert(submission);
    }

    for (&problem_id, submission) in best_submissions.iter() {
        let submission_id = submission.id;
        println!("problem={problem_id}, submission={submission_id}");
        main1(problem_id, submission_id)?;
    }
    Ok(())
}
