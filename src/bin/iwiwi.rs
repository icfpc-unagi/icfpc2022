use icfpc2022;

fn main() {
    let submission_id = std::env::args().nth(1).unwrap();

    let sub: icfpc2022::Submission = serde_json::from_reader(
        std::fs::File::open(format!("submissions/{}.json", submission_id)).unwrap(),
    )
    .unwrap();
    assert_eq!(sub.status, "SUCCEEDED");
    let mut program = icfpc2022::read_isl(
        std::fs::File::open(format!("submissions/{}.isl", submission_id)).unwrap(),
    )
    .unwrap();
    let png = icfpc2022::read_png(&format!("problems/{}.png", sub.problem_id));

    while let Some(improved_program) =
        icfpc2022::local_optimization::optimize_step(program.clone(), &png)
    {
        program = improved_program;
    }

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
}
