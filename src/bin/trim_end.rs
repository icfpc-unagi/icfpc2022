use std::fs::File;

use icfpc2022::{read_isl, read_png, write_isl, Canvas};

fn main1(problem_id: u32, id: &str) -> anyhow::Result<()> {
    let mut f = File::open(format!("submissions/{id}.isl"))?;
    // let id = format!("{problem_id}-{id}");

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

    dbg!(format!("probrems/{problem_id}.png"));
    let answer = read_png(&format!("problems/{problem_id}.png"));
    let score = Canvas::new400().apply_all_and_score(program.clone(), &answer)?;
    let mut f_out = File::create(format!("out/opt_{}_{:06.0}", problem_id, score))?;
    write_isl(&mut f_out, program)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, (u32, String)>::new();

    // from local_optimize_all.rs
    for json_path in glob::glob("submissions/*.json")? {
        let json_path = json_path?;
        let submission_id = json_path.file_name().unwrap().to_str().unwrap();
        let submission_id = submission_id[..submission_id.find('.').unwrap()].to_string();

        let submission: icfpc2022::Submission =
            serde_json::from_reader(std::fs::File::open(json_path)?)?;

        let val = (submission.cost, submission_id);

        best_submissions
            .entry(submission.problem_id)
            .and_modify(|s| {
                if s.0 > val.0 {
                    *s = val.clone()
                }
            })
            .or_insert(val);
    }
    for (&problem_id, (_, submission_id)) in best_submissions.iter() {
        println!("problem={problem_id}, submission={submission_id}");
        main1(problem_id, &submission_id)?;
    }
    Ok(())
}
