use std::{fs::File, io};

use icfpc2022::{flatten_png_data, read_isl, write_png, Canvas, Move};

fn main() -> anyhow::Result<()> {
    let mut f = File::open("submissions/7600.isl")?;
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
        io::BufWriter::new(File::create("tmp/b-animated.png")?),
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
    write_png("tmp/b.png", canvas.bitmap)?;
    Ok(())
}
