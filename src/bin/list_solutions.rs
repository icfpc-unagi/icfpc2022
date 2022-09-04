use icfpc2022::*;

fn main() -> anyhow::Result<()> {
    let submission_ids = submissions::find_all_submission_ids()?;

    for submission_id in submission_ids {
        let (submission, program, comment) = submissions::read_submission_program(submission_id)?;
        let program_name = submissions::estimate_program_name(&comment);

        println!(
            "{:10} {:3} {:10} {}",
            submission.id,
            submission.problem_id,
            submission.cost,
            comment.first().unwrap_or(&"".to_owned())
        )
    }

    Ok(())
}
