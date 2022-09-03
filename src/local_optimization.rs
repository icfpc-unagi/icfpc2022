use super::{Color, Program};
use crate::{Canvas, Move};

const WIDTH: i32 = 400;

pub fn optimize_step(program: Program, image: &Vec<Vec<Color>>) -> Option<Program> {
    let original_score = Canvas::new400()
        .apply_all_and_score(program.clone(), image)
        .unwrap();

    // TODO: random order
    for i in 0..program.len() {
        if let Move::LineCut(bid, ori, offset) = &program[i] {
            for d in [-1, 1] {
                if 0 < offset + d && offset + d < WIDTH {
                    let mut new_program = program.clone();
                    new_program[i] = Move::LineCut(bid.clone(), *ori, offset + d);
                    if let Ok(new_score) =
                        Canvas::new400().apply_all_and_score(new_program.clone(), image)
                    {
                        if new_score < original_score {
                            println!("Improve: {} -> {}", original_score, new_score);
                            return Some(new_program);
                        }
                    }
                }
            }
        }
    }

    None
}
