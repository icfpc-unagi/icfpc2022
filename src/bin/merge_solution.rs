use icfpc2022::{submissions::find_best_submission, *};

fn main() {
    let from = [5, 2, 10, 18, 11, 24, 9, 15, 7, 25];
    for i in 0..10 {
        let problem_id = (26 + i) as u32;
        let (mut init_canvas, png) = load_problem(problem_id);
        let best = find_best_submission(from[i]).unwrap().id;
        let s1 = read_isl(std::fs::File::open(format!("out/tos_merge{}.isl", problem_id)).unwrap())
            .unwrap();
        let s2 =
            read_isl(std::fs::File::open(format!("submissions/{}.isl", best)).unwrap()).unwrap();
        let merged = wata::merge_solution(&init_canvas, &s1, &s2);
        eprintln!(
            "{}: {}",
            problem_id,
            init_canvas
                .apply_all_and_score(merged.clone(), &png)
                .unwrap()
        );
        write_isl_with_comments(
            std::fs::File::create(format!("out/{}.isl", problem_id)).unwrap(),
            merged,
            vec![format!("merged {}", best)],
        )
        .unwrap();
    }
}
