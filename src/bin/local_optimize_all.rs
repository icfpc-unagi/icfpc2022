fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, icfpc2022::Submission>::new();

    for json_path in glob::glob("submissions/*.json")? {
        let submission: icfpc2022::Submission =
            serde_json::from_reader(std::fs::File::open(json_path?)?)?;

        best_submissions
            .entry(submission.problem_id)
            .and_modify(|s| {
                if s.cost > submission.cost {
                    *s = submission.clone()
                }
            })
            .or_insert(submission);
    }

    for (problem_id, submission) in best_submissions.iter() {
        // if *problem_id == 1 {
        //     continue;
        // }

        let (submission, program, image) =
            icfpc2022::local_optimization::read_submission(submission.id)?;
        let (program, score) = icfpc2022::local_optimization::optimize(program, &image, 10, true);

        println!(
            "Problem {:3}: {:7} -> {:7}",
            problem_id, submission.cost, score
        );

        icfpc2022::write_isl(
            std::fs::File::create(format!("out/opt_{}_{:06.0}", submission.problem_id, score))?,
            program,
        )?;
    }

    anyhow::Ok(())
}
