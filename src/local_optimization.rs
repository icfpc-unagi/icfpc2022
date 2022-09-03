use super::{Color, Program};
use crate::{Canvas, Move};

const WIDTH: i32 = 400;

pub fn optimize_step(
    program: Program,
    image: &Vec<Vec<Color>>,
    diff_steps: &[i32],
) -> Option<(Program, f64)> {
    let original_score = Canvas::new400()
        .apply_all_and_score(program.clone(), image)
        .unwrap();

    // TODO: random order
    for i in 0..program.len() {
        if let Move::LineCut(bid, ori, offset) = &program[i] {
            for d in diff_steps {
                if 0 < offset + *d && offset + *d < WIDTH {
                    let mut new_program = program.clone();
                    new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);
                    if let Ok(new_score) =
                        Canvas::new400().apply_all_and_score(new_program.clone(), image)
                    {
                        if new_score < original_score {
                            println!("Improve: {} -> {}", original_score, new_score);
                            return Some((new_program, new_score));
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn optimize(
    mut program: Program,
    image: &Vec<Vec<Color>>,
    max_diff_step: i32,
) -> (Program, f64) {
    let mut result = (
        program.clone(),
        Canvas::new400()
            .apply_all_and_score(program.clone(), image)
            .unwrap(),
    );

    let mut diff_step = 1;
    while diff_step <= max_diff_step {
        let ret = optimize_step(program.clone(), &image, &[-diff_step, diff_step]);

        if let Some((improved_program, improved_score)) = ret {
            program = improved_program.clone();
            result = (improved_program, improved_score);

            // TODO: 時間かかるならこれ
            // icfpc2022::write_isl(
            //     std::fs::File::create(format!(
            //         "out/opt_{}_{:06.0}",
            //         sub.problem_id, improved_score
            //     ))
            //     .unwrap(),
            //     improved_program,
            // )
            // .unwrap();

            diff_step = 1;
        } else {
            println!("Step: {} -> {}", diff_step, diff_step + 1);
            diff_step += 1;
        }
    }

    result
}
