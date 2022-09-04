use std::fs::File;

use icfpc2022::{load_problem, read_isl, write_apng_from_program, write_png};

fn main1(problem_id: u32, id: &str) -> anyhow::Result<()> {
    let mut f = File::open(format!("submissions/{id}.isl"))?;
    let id = format!("{problem_id}-{id}");
    // let mut buf = vec![];
    // f.read_to_end(&mut buf)?;
    // let s = String::from_utf8(buf)?;
    // eprintln!("{}", s);

    let program = read_isl(&mut f)?;
    let mut canvas = load_problem(problem_id).0;
    write_apng_from_program(format!("tmp/{id}-animated.png"), &mut canvas, program, 5)?;
    write_png(&format!("tmp/{id}.png"), canvas.bitmap)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, (u32, String)>::new();

    // from optimize
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
