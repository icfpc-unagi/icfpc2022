use crate::{Move, Program, Submission};

pub fn find_best_submissions() -> anyhow::Result<std::collections::BTreeMap<u32, Submission>> {
    let mut best_submissions = std::collections::BTreeMap::<u32, Submission>::new();

    for json_path in glob::glob("submissions/*.json")? {
        let submission: Submission = serde_json::from_reader(std::fs::File::open(json_path?)?)?;

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

pub fn find_best_submission(problem_id: u32) -> anyhow::Result<Submission> {
    find_best_submissions()?
        .remove(&problem_id)
        .ok_or(anyhow::anyhow!(
            "No submission found for problem: {}",
            problem_id,
        ))
}

fn find_first_cut(program: &Program, orientation: char) -> Option<i32> {
    // TODO: point cut使ったら知らんぞ！

    for mov in program {
        match mov {
            Move::LineCut(_, ori, off)
                if ori.to_ascii_lowercase() == orientation.to_ascii_lowercase() =>
            {
                return Some(*off)
            }
            _ => {}
        }
    }

    None
}

/// (横filpが必要か、縦flipが必要か) で返す
pub fn find_best_flip(problem_id: u32) -> anyhow::Result<(bool, bool)> {
    let submission = find_best_submission(problem_id)?;
    let program = crate::read_isl(std::fs::File::open(format!(
        "submissions/{}.isl",
        submission.id
    ))?)?;

    let x = find_first_cut(&program, 'x').ok_or_else(|| anyhow::anyhow!("No cut found for x"))?;
    let y = find_first_cut(&program, 'y').ok_or_else(|| anyhow::anyhow!("No cut found for y"))?;

    let flip_x = (400 - x) < x;
    let flip_y = (400 - y) < y;

    Ok((flip_x, flip_y))
}
