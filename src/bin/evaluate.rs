use std::fs::File;

use icfpc2022::*;
use serde::{Deserialize, Serialize};

fn main() {
    let problem_id = std::env::args()
        .nth(1)
        .expect("Problem ID must be specified in the first argument.");
    let isl_path = std::env::args()
        .nth(2)
        .expect("Path to ISL must be specified in the second argument.");

    let isl = read_isl(File::open(isl_path).unwrap()).unwrap();
    let png = read_png(&format!("problems/{}.png", problem_id));
    let mut canvas = Canvas::new400();
    let cost = canvas.apply_all(isl).round() as u64;
    let similarity = similarity(&png, &canvas.bitmap).round() as u64;

    let mut image = Vec::<u8>::new();

    {
        let mut encoder = png::Encoder::new(
            &mut image,
            canvas.bitmap[0].len() as u32,
            canvas.bitmap.len() as u32,
        );
        encoder.set_compression(png::Compression::Best);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let data = Vec::from_iter(canvas.bitmap.iter().rev().flatten().flatten().cloned());
        writer.write_image_data(&data).unwrap();
    }

    #[derive(Serialize, Deserialize)]
    struct Response {
        problem_id: u32,
        cost: u64,
        similarity: u64,
        image: String,
    }

    println!(
        "{}",
        serde_json::to_string(&Response {
            problem_id: problem_id.parse().unwrap(),
            cost: cost,
            similarity: similarity,
            image: base64::encode(image),
        })
        .unwrap()
    );
}
