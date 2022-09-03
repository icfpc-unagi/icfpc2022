use icfpc2022::*;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;
use svg::node::{element::{Group, Title, Rectangle, Image}, Text};

#[allow(unused)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const PNG: [&'static [u8]; 30] = [
    include_bytes!("../../problems/1.png"),
    include_bytes!("../../problems/2.png"),
    include_bytes!("../../problems/3.png"),
    include_bytes!("../../problems/4.png"),
    include_bytes!("../../problems/5.png"),
    include_bytes!("../../problems/6.png"),
    include_bytes!("../../problems/7.png"),
    include_bytes!("../../problems/8.png"),
    include_bytes!("../../problems/9.png"),
    include_bytes!("../../problems/10.png"),
    include_bytes!("../../problems/11.png"),
    include_bytes!("../../problems/12.png"),
    include_bytes!("../../problems/13.png"),
    include_bytes!("../../problems/14.png"),
    include_bytes!("../../problems/15.png"),
    include_bytes!("../../problems/16.png"),
    include_bytes!("../../problems/17.png"),
    include_bytes!("../../problems/18.png"),
    include_bytes!("../../problems/19.png"),
    include_bytes!("../../problems/20.png"),
    include_bytes!("../../problems/21.png"),
    include_bytes!("../../problems/22.png"),
    include_bytes!("../../problems/23.png"),
    include_bytes!("../../problems/24.png"),
    include_bytes!("../../problems/25.png"),
    include_bytes!("../../problems/26.png"),
    include_bytes!("../../problems/27.png"),
    include_bytes!("../../problems/28.png"),
    include_bytes!("../../problems/29.png"),
    include_bytes!("../../problems/30.png"),
];

const INIT_CANVAS: [Option<&'static [u8]>; 30] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(include_bytes!("../../problems/26.initial.json")),
    Some(include_bytes!("../../problems/27.initial.json")),
    Some(include_bytes!("../../problems/28.initial.json")),
    Some(include_bytes!("../../problems/29.initial.json")),
    Some(include_bytes!("../../problems/30.initial.json")),
];

#[wasm_bindgen]
pub struct Ret {
    pub score: i64,
    #[wasm_bindgen(getter_with_clone)]
    pub error: String,
    #[wasm_bindgen(getter_with_clone)]
    pub svg: String,
}

fn base64(png: &Vec<Vec<[u8; 4]>>) -> String {
    let h = png.len();
    let w = png[0].len();
    let mut writer = vec![];
    let mut encoder = png::Encoder::new(&mut writer, w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut data = vec![];
    for y in (0..h).rev() {
        for x in 0..w {
            for c in 0..4 {
                data.push(png[y][x][c]);
            }
        }
    }
    encoder.write_header().unwrap().write_image_data(&data).unwrap();
    base64::encode(writer)
}

#[wasm_bindgen]
pub fn vis(problem_id: String, output: String, t: i32, show_blocks: bool, show_diff: bool) -> Ret {
    console_error_panic_hook::set_once();
    let problem_id = problem_id.parse::<usize>().unwrap() - 1;
    if problem_id >= PNG.len() {
        return Ret {
            score: 0,
            error: "Illegal problem ID".to_owned(),
            svg: String::new()
        };
    }
    let decoder = png::Decoder::new(PNG[problem_id]);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let h = info.height as usize;
    let w = info.width as usize;
    let mut png = mat![[0; 4]; h; w];
    for i in 0..h {
        for j in 0..w {
            for k in 0..4 {
                png[h - i - 1][j][k] = buf[(i * w + j) * 4 + k];
            }
        }
    }
    let mut score = 0;
    let mut error = String::new();
    let mut doc = svg::Document::new().set("id", "vis").set("viewBox", (-5, -5, w * 4 + 10 + 50, h * 2 + 10)).set("width", w * 4 + 10 + 50).set("height", h * 2 + 10);
    if !show_diff {
        doc = doc.add(Image::new().set("x", w * 2 + 50).set("y", 0).set("width", w * 2).set("height", h * 2).set("xlink:href", format!("data:image/png;base64,{}",base64(&png))));
    }
    match read_isl(output.into_bytes().as_slice()) {
        Ok(program) => {
            let mut canvas =
            if let Some(json) = INIT_CANVAS[problem_id] {
                let json: InitialJson = serde_json::from_reader(json).unwrap();
                Canvas::try_from(json).unwrap()
            } else {
                Canvas::new(png[0].len(), png.len())
            };
            match canvas.apply_all_safe(program[0..t as usize].iter().cloned()) {
                Ok(s) => {
                    score += s.round() as i64
                },
                Err(e) => {
                    error = e.to_string();
                }
            }
            score += similarity(&png, &canvas.bitmap).round() as i64;
            doc = doc.add(Image::new().set("x", 0).set("y", 0).set("width", w * 2).set("height", h * 2).set("xlink:href", format!("data:image/png;base64,{}",base64(&canvas.bitmap))));
            if show_blocks {
                for (id, block) in canvas.blocks.iter() {
                    doc = doc.add(Group::new().add(Title::new().add(Text::new(format!("block {}\n({}, {}) - ({}, {})", id, block.0.0, block.0.1, block.1.0, block.1.1)))).add(Rectangle::new().set("x", block.0.0 * 2).set("y", 2 * h as i32 - block.1.1 * 2).set("width", 2 * (block.1.0 - block.0.0)).set("height", 2 * (block.1.1 - block.0.1)).set("fill", "#00000000").set("stroke-width", 2).set("stroke", "red")));
                }
            }
            if show_diff {
                let mut diff = mat![[255; 4]; h; w];
                for y in 0..h {
                    for x in 0..w {
                        for c in 0..3 {
                            diff[y][x][c] = png[y][x][c].abs_diff(canvas.bitmap[y][x][c]);
                        }
                    }
                }
                doc = doc.add(Image::new().set("x", w * 2 + 50).set("y", 0).set("width", w * 2).set("height", h * 2).set("xlink:href", format!("data:image/png;base64,{}",base64(&diff))));
            }
        },
        Err(err) => {
            error = err.to_string()
        }
    }
    Ret {
        score,
        error,
        svg: doc.to_string()
    }
}

#[wasm_bindgen]
pub fn get_max_turn(output: String) -> i32 {
    console_error_panic_hook::set_once();
    read_isl(output.into_bytes().as_slice()).unwrap().len() as i32
}
