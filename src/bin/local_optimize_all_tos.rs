// initial canvas対応のためライブラリいじった際にコンパイル通らなくなったけど多分使ってない？気がするから一旦コメントアウトします by iwiwi

// fn main_() -> anyhow::Result<()> {
//     #[rustfmt::skip]
//     let done = [
//         7771,
//         11020,
//         11139,
//         11267,
//         9063,
//         11141,
//         11142,
//         11268,
//         11143,
//         11145,
//         11147,
//         11270,
//         11148,
//         11150,
//         11151,
//         11153,
//         11154,
//         11155,
//         11156,
//         11157,
//         11299,
//         11159,
//         11732,
//         11161,
//         11162,
//         // 1st opt results
//         12138,
//         12141,
//         // 2nd runs
//         9024,
//         7858,
//         10268,
//         9066,
//         8082,
//         8112,
//         10691,
//         // 3rd runs
//         11178,
//         11544,
//         12042,
//         12366,
//         // 3rd opt results
//         13263,
//         13269,
//         // #4
//         12999,
//         13031,
//         13218,
//         // #5
//         7822,
//         3982,
//         // #6
//         7890,
//         7943,
//         8182,
//         8338,
//         8460,
//         8535,
//         8581,
//         8683,
//         8691,
//         8751,
//         8849,
//         8991,
//         9039,
//         9093,
//         9269,
//         10584,
//         10145,
//         10027,
//         10270,
//         8090,
//         // #7
//         9618,
//         9902,
//         10592,
//         // #8
//         13411,
//         13665,
//         // #8 opt
//         13954,
//         13957,
//         // #9
//         12759,
//     ];
//     let done = done
//         .iter()
//         .copied()
//         .collect::<std::collections::BTreeSet<_>>();
//     let mut best_submissions = std::collections::BTreeMap::<u32, icfpc2022::Submission>::new();
//     let mut true_best_scores = std::collections::BTreeMap::<u32, u32>::new();
//
//     for json_path in glob::glob("submissions/*.json")? {
//         let submission: icfpc2022::Submission =
//             serde_json::from_reader(std::fs::File::open(json_path?)?)?;
//
//         let e = true_best_scores
//             .entry(submission.problem_id)
//             .or_insert(u32::MAX);
//         *e = submission.cost.min(*e);
//         let submission_id = submission.id;
//         if done.contains(&submission_id)
//             || submission_id <= 7600
//             || (11020 <= submission_id && submission_id <= 11162)
//         {
//             eprintln!("skipping {submission_id}");
//             continue;
//         }
//
//         best_submissions
//             .entry(submission.problem_id)
//             .and_modify(|s| {
//                 if s.cost > submission.cost {
//                     *s = submission.clone()
//                 }
//             })
//             .or_insert(submission);
//     }
//
//     let mut gain = 0.0;
//
//     // dbg!(true_best_scores);
//     // return Ok(());
//
//     // let ignore_problems = std::collections::BTreeSet::from([
//     //     1,
//     // ]);
//     for (problem_id, submission) in best_submissions.iter() {
//         // if *problem_id == 1 {
//         //     continue;
//         // }
//         // if ignore_problems.contains(problem_id) {
//         //     continue;
//         // }
//
//         let true_best_score = true_best_scores[problem_id];
//         println!(
//             "Problem {:3}: submission_id={:6}, score={:7}, true_best={:7}",
//             problem_id, submission.id, submission.cost, true_best_score
//         );
//         let (submission, program, image) =
//             icfpc2022::local_optimization::read_submission(submission.id)?;
//
//         let (program, score) = icfpc2022::local_optimization::optimize(program, &image, 10, true);
//
//         //let program = icfpc2022::local_optimization::optimize_color(program.clone(), &image);
//         //let score = icfpc2022::Canvas::new400().apply_all_and_score(program.clone(), &image)?;
//
//         println!(
//             "Problem {:3}: {:7} ({:7}) -> {:7}",
//             problem_id, submission.cost, true_best_scores[problem_id], score
//         );
//         gain += (true_best_score as f64 - score).max(0.0);
//
//         icfpc2022::write_isl(
//             std::fs::File::create(format!("out/opt_{}_{:06.0}", submission.problem_id, score))?,
//             program,
//         )?;
//     }
//
//     println!("Total gain: {}", gain);
//
//     anyhow::Ok(())
// }
