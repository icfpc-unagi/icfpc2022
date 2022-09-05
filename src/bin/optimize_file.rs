use clap::Parser;
use icfpc2022::*;
use std::io::Write;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    #[clap(long)]
    problem_id: u32,

    #[clap(long)]
    program: std::path::PathBuf,

    #[clap(long, default_value_t = 0)]
    max_pair_perturb: i32,

    #[clap(long, default_value = "optimized.txt")]
    out: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (program, comments) = read_isl_with_comments(std::fs::File::open(args.program)?)?;
    let (initial_canvas, image) = load_problem(args.problem_id);
    let score = canvas::score(&program, &initial_canvas, &image)?;

    println!("ID     Problem Program              Old    New    +-");
    println!("------ ------- -------------------- ------ ------ ------");

    let (new_program, new_score) = local_optimization::optimize(
        program,
        &initial_canvas,
        &image,
        10,
        true,
        args.max_pair_perturb,
    );

    println!(
        "{:6} {:7} {:20} {:6} {:6} {:6}",
        "?",
        args.problem_id,
        submissions::estimate_program_name(&comments),
        score,
        new_score,
        score - new_score,
    );

    let mut w = std::fs::File::create(args.out)?;
    write!(
        &mut w,
        "# optimizeV2 FROM_FILE MAX_PAIR_PERTURB={}\n",
        args.max_pair_perturb
    )?;
    write_isl_with_comments(w, new_program, &comments)?;

    anyhow::Ok(())
}
