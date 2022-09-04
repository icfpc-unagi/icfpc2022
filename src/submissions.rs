use crate::{Canvas, Move, Program, Submission};
use anyhow::Context;
use once_cell::sync::Lazy;
use std::collections::HashSet;

pub static SUBMISSIONS_DIR: Lazy<String> =
    Lazy::new(|| std::env::var("SUBMISSIONS_DIR").unwrap_or("./submissions".to_owned()));

pub fn find_all_submission_ids() -> anyhow::Result<Vec<u32>> {
    let paths = glob::glob(&format!("{}/*.json", &*SUBMISSIONS_DIR))?;
    let paths = paths
        .map(|path| path.map_err(anyhow::Error::from))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut submission_ids = paths
        .into_iter()
        .map(|path| {
            path.with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<u32>()
                .map_err(anyhow::Error::from)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    submission_ids.sort();
    Ok(submission_ids)
}

pub fn estimate_program_name(comment: &Vec<String>) -> String {
    if comment.len() == 0 {
        return "".to_owned();
    } else {
        return comment[0].split_whitespace().nth(0).unwrap().to_owned();
    }
}

pub fn read_submission(submission_id: u32) -> anyhow::Result<Submission> {
    let sub: Submission = serde_json::from_reader(std::fs::File::open(format!(
        "{}/{}.json",
        &*SUBMISSIONS_DIR, submission_id
    ))?)?;
    if sub.status.as_ref().map_or(false, |s| s != "SUCCEEDED") {
        anyhow::bail!("Submission status is not SUCCEEDED {:?}", &sub);
    }
    anyhow::Ok(sub)
}

pub fn read_all_submissions_and_programs() -> anyhow::Result<Vec<(Submission, Program, Vec<String>)>>
{
    let mut submission_ids = find_all_submission_ids()?;
    submission_ids.sort();
    submission_ids.reverse();
    submission_ids
        .into_iter()
        .map(|submission_id| read_submission_program(submission_id))
        .collect::<anyhow::Result<Vec<_>>>()
}

pub fn find_best_submissions() -> anyhow::Result<std::collections::BTreeMap<u32, Submission>> {
    let mut best_submissions = std::collections::BTreeMap::<u32, Submission>::new();

    for json_path in glob::glob(&format!("{}/*.json", &*SUBMISSIONS_DIR))? {
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
        "{}/{}.isl",
        &*SUBMISSIONS_DIR, submission.id
    ))?)?;

    let x = find_first_cut(&program, 'x').ok_or_else(|| anyhow::anyhow!("No cut found for x"))?;
    let y = find_first_cut(&program, 'y').ok_or_else(|| anyhow::anyhow!("No cut found for y"))?;

    let flip_x = (400 - x) < x;
    let flip_y = (400 - y) < y;

    Ok((flip_x, flip_y))
}

pub fn find_best_score(problem_id: u32) -> u32 {
    if let Ok(s) = find_best_submission(problem_id) {
        s.cost
    } else {
        u32::MAX
    }
}

pub fn read_submission_program(
    submission_id: u32,
) -> anyhow::Result<(Submission, Program, Vec<String>)> {
    let submission: Submission = serde_json::from_reader(std::fs::File::open(format!(
        "{}/{}.json",
        &*SUBMISSIONS_DIR, submission_id
    ))?)
    .with_context(|| format!("Error parsing {}.json", submission_id))?;
    if submission
        .status
        .as_ref()
        .map_or(false, |s| s != "SUCCEEDED")
    {
        anyhow::bail!("Submission status is not SUCCEEDED {:?}", &submission);
    }
    // dbg!(submission_id);
    let (program, comments) = crate::read_isl_with_comments(std::fs::File::open(format!(
        "{}/{}.isl",
        &*SUBMISSIONS_DIR, submission_id
    ))?)?;

    Ok((submission, program, comments))
}

pub fn read_submission_program_problem(
    submission_id: u32,
) -> anyhow::Result<(
    Submission,
    Program,
    Vec<String>,
    Canvas,
    Vec<Vec<crate::Color>>,
)> {
    let (submission, program, comments) = read_submission_program(submission_id)?;
    let (initial_canvas, image) = crate::load_problem(submission.problem_id);
    Ok((submission, program, comments, initial_canvas, image))
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// CLIでいろんな条件でsubmissionを指定したくなってきているので、そのためのユーティリティ
pub fn query_submission_ids(
    latest: Option<usize>,
    problem_ids: Option<String>,
    submission_ids: Option<String>,
    submission_id_min: Option<u32>,
    program_name: Option<String>,
    allow_not_best: bool,
) -> anyhow::Result<Vec<(Submission, Program, Vec<String>)>> {
    let mut spcs = read_all_submissions_and_programs()?;
    eprintln!("All submissions: {}", spcs.len());

    if let Some(latest) = latest {
        spcs.truncate(latest)
    }

    if let Some(problem_ids_str) = problem_ids {
        let problem_ids = problem_ids_str
            .split_whitespace()
            .map(|id_str: &str| id_str.trim().parse::<u32>().map_err(anyhow::Error::from))
            .collect::<anyhow::Result<HashSet<_>>>()?;

        spcs = spcs
            .into_iter()
            .filter(|(s, _, _)| problem_ids.contains(&s.problem_id))
            .collect();

        eprintln!("Submissions fitlered by problem IDs: {}", spcs.len());
    }

    if let Some(submission_ids_str) = submission_ids {
        let submission_ids = submission_ids_str
            .split_whitespace()
            .map(|id_str: &str| id_str.trim().parse::<u32>().map_err(anyhow::Error::from))
            .collect::<anyhow::Result<HashSet<_>>>()?;

        spcs = spcs
            .into_iter()
            .filter(|(s, _, _)| submission_ids.contains(&s.id))
            .collect();

        eprintln!("Submissions filtered by submission IDs: {}", spcs.len());
    }

    if let Some(submission_id_min) = submission_id_min {
        spcs = spcs
            .into_iter()
            .filter(|(submission, _, _)| submission.id >= submission_id_min)
            .collect();

        eprintln!("Submissions filtered by submission ID min: {}", spcs.len());
    }

    if let Some(program_name) = program_name {
        spcs = spcs
            .into_iter()
            .filter(|(_, _, c)| estimate_program_name(&c) == program_name)
            .collect();
        eprintln!("Submissions filtered by program name: {}", spcs.len());
    }

    if !allow_not_best {
        let mut old_spcs = vec![];
        std::mem::swap(&mut spcs, &mut old_spcs);
        let mut best_submissions =
            std::collections::BTreeMap::<u32, (Submission, Program, Vec<String>)>::new();
        for spc in old_spcs {
            best_submissions
                .entry(spc.0.problem_id)
                .and_modify(|best_spc| {
                    if best_spc.0.cost > spc.0.cost {
                        *best_spc = spc.clone()
                    }
                })
                .or_insert(spc);
        }
        spcs = best_submissions.into_values().collect();

        eprintln!("Submissions filtered by best per problem: {}", spcs.len());
    }

    // let submissions: Vec<_> = best_submissions.into_values().collect();
    // println!("Applying to {} submissions", submissions.len());

    Ok(spcs)
}
