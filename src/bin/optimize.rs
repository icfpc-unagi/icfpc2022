use std::io::Write;

use clap::Parser;
use icfpc2022::*;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    #[clap(short, long)]
    problem_ids: Option<String>,

    #[clap(short, long)]
    submission_ids: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let submissions: Vec<_>;
    if let Some(submission_ids) = args.submission_ids {
        if args.problem_ids.is_some() {
            anyhow::bail!("Do not specify `--problem-ids` and `--submission-ids` at the same time")
        }
        submissions = submission_ids
            .split_whitespace()
            .map(|id_str: &str| {
                id_str
                    .trim()
                    .parse::<u32>()
                    .map_err(anyhow::Error::from)
                    .and_then(|id_u32| submissions::read_submission(id_u32))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
    } else if let Some(problem_ids) = args.problem_ids {
        submissions = problem_ids
            .split_whitespace()
            .map(|id_str: &str| {
                id_str
                    .trim()
                    .parse::<u32>()
                    .map_err(anyhow::Error::from)
                    .and_then(|id_u32| submissions::find_best_submission(id_u32))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
    } else {
        submissions = submissions::find_best_submissions()?
            .into_values()
            .collect();
    }
    println!("Applying to {} submissions", submissions.len());

    let mut gain = 0.0;
    for submission in submissions {
        let (submission, program, comments, initial_canvas, image) =
            submissions::read_solution(submission.id)?;

        let (program, score) =
            local_optimization::optimize(program, &initial_canvas, &image, 10, true);

        // let (program, score) =
        //     icfpc2022::local_optimization::try_removing_color_op(program.clone(), &image);

        //let program = icfpc2022::local_optimization::optimize_color(program.clone(), &image);
        // let score = icfpc2022::Canvas::new400().apply_all_and_score(program.clone(), &image)?;

        println!(
            "Problem {:3}: {:7} -> {:7}",
            submission.problem_id, submission.cost, score
        );
        gain += submission.cost as f64 - score;

        let mut w =
            std::fs::File::create(format!("out/opt_{}_{:06.0}", submission.problem_id, score))?;
        w.write_fmt(format_args!("# optimize\n"))?;
        write_isl_with_comments(w, program, &comments)?;
    }

    println!("Total gain: {}", gain);
    anyhow::Ok(())
}
