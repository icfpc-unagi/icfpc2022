pub fn find_best_submissions() -> anyhow::Result<std::collections::BTreeMap<u32, super::Submission>>
{
    let mut best_submissions = std::collections::BTreeMap::<u32, super::Submission>::new();

    for json_path in glob::glob("submissions/*.json")? {
        let submission: super::Submission =
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

    anyhow::Ok(best_submissions)
}
