use icfpc2022::submissions;

fn main() -> anyhow::Result<()> {
    for problem_id in 1..=25 {
        let (flip_x, flip_y) = submissions::find_best_flip(problem_id)?;
        println!(
            "Problem {:2}: flip_x={}, flip_y={}",
            problem_id, flip_x, flip_y
        )
    }

    anyhow::Ok(())
}
