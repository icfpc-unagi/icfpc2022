use icfpc2022;
use icfpc2022::Canvas;

fn main() -> anyhow::Result<()> {
    let submission_id: u32 = std::env::args().nth(1).unwrap().parse()?;
    let (submission, program, image) =
        icfpc2022::local_optimization::read_submission(submission_id)?;

    let (program, score) = icfpc2022::local_optimization::optimize(program, &image, 10);
    icfpc2022::write_isl(
        std::fs::File::create(format!("out/opt_{}_{:06.0}", submission.problem_id, score)).unwrap(),
        program,
    )
    .unwrap();

    // let mut step = 1;
    // loop {
    //     let result =
    //         icfpc2022::local_optimization::optimize_step(program.clone(), &png, &[-step, step]);
    //
    //     if let Some((improved_program, improved_score)) = result {
    //         program = improved_program.clone();
    //         icfpc2022::write_isl(
    //             std::fs::File::create(format!(
    //                 "out/opt_{}_{:06.0}",
    //                 sub.problem_id, improved_score
    //             ))
    //             .unwrap(),
    //             improved_program,
    //         )
    //         .unwrap();
    //
    //         step = 1;
    //     } else {
    //         println!("Step: {} -> {}", step, step + 1);
    //         step += 1;
    //     }
    // }
    //
    // while let Some((improved_program, improved_score)) =
    //     icfpc2022::local_optimization::optimize_step(program.clone(), &png, &[-2, 2])
    // {
    //     program = improved_program.clone();
    //     icfpc2022::write_isl(
    //         std::fs::File::create(format!(
    //             "out/opt_{}_{:06.0}",
    //             sub.problem_id, improved_score
    //         ))
    //         .unwrap(),
    //         improved_program,
    //     )
    //     .unwrap();
    // }

    // assert_eq!()

    // let program = icfpc2022::read_isl(std::io::stdin()).unwrap();
    // let mut canvas = icfpc2022::Canvas::new400();
    // canvas.apply_all(program.into_iter());

    // let program = vec![icfpc2022::Move::LineCut(
    //     icfpc2022::BlockId(vec![0]),
    //     'x',
    //     0,
    // )];
    //
    // // canvas.apply_all()
    //
    // canvas.apply_all(program.into_iter());

    //let input = std::env::args().nth(1).unwrap();
    //let png = icfpc2022::read_png(&input);

    // // let program = vec![icfpc2022::Move::Color(icfpc::BlockId(vec![0]))];
    // let program = icfpc2022::read_isl(std::io::stdin()).unwrap();
    // // dbg!(&program);
    //
    // //let program2 = icfpc2022::rotate::rotate_program(&program);
    // //dbg!(&program2);
    //
    // let program2 = icfpc2022::rotate::flip_program(&program);
    // //dbg!(&program2);
    //
    // let program2 = icfpc2022::rotate::rotate_program(&program);
    // // icfpc2022::write_isl(std::io::stdout(), program2).unwrap();
    //
    // let mut canvas = icfpc2022::Canvas::default();
    // canvas.apply_all(program2.into_iter());

    Ok(())
}
