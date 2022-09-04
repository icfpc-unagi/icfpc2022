use icfpc2022::{load_problem, optmerge::merge_all};

fn main() {
    let (mut canvas, _) = load_problem(26);
    merge_all(&mut canvas);
}
