use clap::Parser;
use icfpc2022::*;
use std::io::Write;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    #[clap(short, long)]
    latest: Option<usize>,

    #[clap(short, long)]
    problem_ids: Option<String>,

    #[clap(short, long)]
    submission_ids: Option<String>,

    #[clap(short, long)]
    submission_id_min: Option<u32>,

    #[clap(short, long)]
    program_name: Option<String>,

    #[clap(short, long)]
    allow_not_best: bool,

    #[clap(short, long)]
    dryrun: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let spcs = submissions::query_submission_ids(
        args.latest,
        args.problem_ids,
        args.submission_ids,
        args.submission_id_min,
        args.program_name,
        args.allow_not_best,
    )?;
    println!("Applying to {} submissions", spcs.len());

    let mut gain = 0.0;
    println!("ID     Problem Program              Old    New    +-");
    println!("------ ------- -------------------- ------ ------ ------");
    for (submission, _, _) in spcs {
        let (submission, program, comments, initial_canvas, image) =
            submissions::read_submission_program_problem(submission.id)?;

        let (new_program, new_score) = if args.dryrun {
            (program.clone(), submission.cost as f64)
        } else {
            local_optimization::optimize(program, &initial_canvas, &image, 10, true)
        };

        println!(
            "{:6} {:7} {:20} {:6} {:6} {:6}",
            submission.id,
            submission.problem_id,
            submissions::estimate_program_name(&comments),
            submission.cost,
            new_score,
            submission.cost as f64 - new_score,
        );

        gain += submission.cost as f64 - new_score;

        let mut w = std::fs::File::create(format!(
            "out/opt_{}_{:06.0}",
            submission.problem_id, new_score
        ))?;
        w.write_fmt(format_args!("# optimize\n"))?;
        write_isl_with_comments(w, new_program, &comments)?;
    }

    println!("Total gain: {}", gain);

    anyhow::Ok(())
}
