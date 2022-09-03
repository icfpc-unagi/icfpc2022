use std::{fs::File, io};

use icfpc2022::{flatten_png_data, read_isl, write_png, Canvas, Move};

fn main1(id: &str) -> anyhow::Result<()> {
    let mut f = File::open(format!("submissions/{id}.isl"))?;
    // let mut buf = vec![];
    // f.read_to_end(&mut buf)?;
    // let s = String::from_utf8(buf)?;
    // eprintln!("{}", s);

    let program = read_isl(&mut f)?;
    let n_frames = 1 + program
        .iter()
        .filter(|m| matches!(m, Move::Color(_, _) | Move::Swap(_, _)))
        .count(); // program.len() + 1;

    let mut encoder = png::Encoder::new(
        io::BufWriter::new(File::create(format!("tmp/{id}-animated.png"))?),
        400,
        400,
    );
    encoder.set_animated(n_frames as u32, 0)?;
    encoder.set_frame_delay(5, n_frames as u16)?;
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let mut canvas = Canvas::new400();

    {
        let bitmap = &canvas.bitmap;
        let data = Vec::from_iter(bitmap.iter().rev().flatten().flatten().cloned());
        writer.write_image_data(&data)?;
    }

    for m in program.iter() {
        canvas.apply(m);
        if matches!(m, Move::Color(_, _) | Move::Swap(_, _)) {
            let data = flatten_png_data(&canvas.bitmap);
            writer.write_image_data(&data)?;
        }
    }
    // eprintln!("{:?}", program);
    write_png(&format!("tmp/{id}.png"), canvas.bitmap)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut best_submissions = std::collections::BTreeMap::<u32, (u32, String)>::new();

    // from local_optimize_all.rs
    for json_path in glob::glob("submissions/*.json")? {
        let json_path = json_path?;
        let submission_id = json_path.file_name().unwrap().to_str().unwrap();
        let submission_id = submission_id[..submission_id.find('.').unwrap()].to_string();

        let submission: icfpc2022::Submission =
            serde_json::from_reader(std::fs::File::open(json_path)?)?;

        let val = (submission.cost, submission_id);

        best_submissions
            .entry(submission.problem_id)
            .and_modify(|s| {
                if s.0 > val.0 {
                    *s = val.clone()
                }
            })
            .or_insert(val);
    }
    for (&problem_id, (_, submission_id)) in best_submissions.iter() {
        println!("problem={problem_id}, submission={submission_id}");
        main1(&submission_id);
    }
    Ok(())
}
