use std::{fs::File, io};

use icfpc2022::read_png;

fn main() {
    let mut encoder = png::Encoder::new(
        io::BufWriter::new(File::create("tmp/a.png").unwrap()),
        400,
        400,
    );
    encoder.set_animated(3, 0).unwrap();
    encoder.set_frame_delay(1, 3).unwrap();
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    for i in [9, 13, 16] {
        let input_path = format!("problems/{i}.png");
        let bitmap = read_png(&input_path);
        let data = Vec::from_iter(bitmap.iter().rev().flatten().flatten().cloned());
        writer.write_image_data(&data).unwrap();
    }
}
