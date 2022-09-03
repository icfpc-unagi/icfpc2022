fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, icfpc2022::Submission>::new();

    for json_path in glob::glob("submissions/*.json")? {
        let submission: icfpc2022::Submission =
            serde_json::from_reader(std::fs::File::open(json_path?)?)?;

        best_submissions
            .entry(submission.problem_id)
            .and_modify(|s| {
                if s.score > submission.score {
                    *s = submission.clone()
                }
            })
            .or_insert(submission);
    }

    for (problem_id, submission) in best_submissions.iter() {
        println!("{} {:?}", problem_id, submission)
    }

    anyhow::Ok(())
}
