fn main() -> anyhow::Result<()> {
    let best_submissions = icfpc2022::submissions::find_best_submissions()?;

    let mut gain = 0.0;

    for (problem_id, submission) in best_submissions.iter() {
        if *problem_id >= 26 {
            continue;
        }

        let (submission, program, image) =
            icfpc2022::local_optimization::read_submission(submission.id)?;
        //println!("{}", submission.id);
        let score = icfpc2022::canvas::score(&program, &image)?;
        let (program, score) = icfpc2022::local_optimization::optimize(program, &image, 10, true);

        // let (program, score) =
        //     icfpc2022::local_optimization::try_removing_color_op(program.clone(), &image);

        //let program = icfpc2022::local_optimization::optimize_color(program.clone(), &image);
        // let score = icfpc2022::Canvas::new400().apply_all_and_score(program.clone(), &image)?;

        println!(
            "Problem {:3}: {:7} -> {:7}",
            problem_id, submission.cost, score
        );
        gain += submission.cost as f64 - score;

        icfpc2022::write_isl(
            std::fs::File::create(format!("out/opt_{}_{:06.0}", submission.problem_id, score))?,
            program,
        )?;
    }

    println!("Total gain: {}", gain);

    anyhow::Ok(())
}
