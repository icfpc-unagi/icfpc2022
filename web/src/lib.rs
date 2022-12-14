use icfpc2022::*;
use std::convert::TryFrom;
use svg::node::{
    element::{Group, Image, Rectangle, Title},
    Text,
};
use wasm_bindgen::prelude::*;
use std::fmt::Write;

#[allow(unused)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const PNG: [&'static [u8]; 40] = [
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
    include_bytes!("../../problems/31.png"),
    include_bytes!("../../problems/32.png"),
    include_bytes!("../../problems/33.png"),
    include_bytes!("../../problems/34.png"),
    include_bytes!("../../problems/35.png"),
    include_bytes!("../../problems/36.png"),
    include_bytes!("../../problems/37.png"),
    include_bytes!("../../problems/38.png"),
    include_bytes!("../../problems/39.png"),
    include_bytes!("../../problems/40.png"),
];

const INIT_CANVAS: [Option<&'static [u8]>; 40] = [
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
    Some(include_bytes!("../../problems/31.initial.json")),
    Some(include_bytes!("../../problems/32.initial.json")),
    Some(include_bytes!("../../problems/33.initial.json")),
    Some(include_bytes!("../../problems/34.initial.json")),
    Some(include_bytes!("../../problems/35.initial.json")),
    Some(include_bytes!("../../problems/36.initial.json")),
    Some(include_bytes!("../../problems/37.initial.json")),
    Some(include_bytes!("../../problems/38.initial.json")),
    Some(include_bytes!("../../problems/39.initial.json")),
    Some(include_bytes!("../../problems/40.initial.json")),
];

const INIT_PNG: [&'static [u8]; 40] = [
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    include_bytes!("../../problems/36.initial.png"),
    include_bytes!("../../problems/37.initial.png"),
    include_bytes!("../../problems/38.initial.png"),
    include_bytes!("../../problems/39.initial.png"),
    include_bytes!("../../problems/40.initial.png"),
];

#[wasm_bindgen]
pub struct Ret {
    #[wasm_bindgen(getter_with_clone)]
    pub score: String,
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
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&data)
        .unwrap();
    base64::encode(writer)
}

fn read_png(data: &[u8]) -> Vec<Vec<[u8; 4]>> {
    let decoder = png::Decoder::new(data);
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
    png
}

#[wasm_bindgen]
pub fn vis(problem_id: String, output: String, t: i32, show_blocks: bool, show_diff: bool, swap_input: bool) -> Ret {
    console_error_panic_hook::set_once();
    let problem_id = problem_id.parse::<usize>().unwrap() - 1;
    if problem_id >= PNG.len() {
        return Ret {
            score: String::new(),
            error: "Illegal problem ID".to_owned(),
            svg: String::new(),
        };
    }
    let mut png = read_png(PNG[problem_id]);
    let h = png.len();
    let w = png[0].len();
    let mut cost = 0;
    let mut similarity = 0;
    let mut error = String::new();
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("viewBox", (-5, -5, w * 4 + 10 + 50, h * 2 + 10))
        .set("width", w * 4 + 10 + 50)
        .set("height", h * 2 + 10);
    match read_isl(output.into_bytes().as_slice()) {
        Ok(program) => {
            let mut dummy_canvas = Canvas::new(w, h);
            let _ = dummy_canvas.apply_all_safe(program[0..t as usize].iter().cloned());
            let mut canvas = if let Some(json) = INIT_CANVAS[problem_id] {
                let mut json: InitialJson = serde_json::from_reader(json).unwrap();
                if problem_id >= 35 {
                    json.source_png_p_n_g = None;
                    json.blocks[0].color = Some([0, 0, 0, 0]);
                }
                let mut canvas = Canvas::try_from(json).unwrap();
                if problem_id >= 35 {
                    canvas.bitmap = read_png(&INIT_PNG[problem_id]);
                    canvas.cost_type = CostType::V2;
                }
                canvas
            } else {
                Canvas::new(png[0].len(), png.len())
            };
            if swap_input {
                canvas.bitmap = png.clone();
            }
            png = wata::get_swapped_png(&png, &program[t as usize..], &dummy_canvas);
            match canvas.apply_all_safe(program[0..t as usize].iter().cloned()) {
                Ok(s) => cost = s.round() as i64,
                Err(e) => {
                    error = e.to_string();
                }
            }
            similarity = icfpc2022::similarity(&png, &canvas.bitmap).round() as i64;
            doc = doc.add(
                Image::new()
                    .set("x", 0)
                    .set("y", 0)
                    .set("width", w * 2)
                    .set("height", h * 2)
                    .set(
                        "xlink:href",
                        format!("data:image/png;base64,{}", base64(&canvas.bitmap)),
                    ),
            );
            if show_diff {
                let diff = pixel_distance_bitmap(&png, &canvas.bitmap);
                doc = doc.add(
                    Image::new()
                        .set("x", w * 2 + 50)
                        .set("y", 0)
                        .set("width", w * 2)
                        .set("height", h * 2)
                        .set(
                            "xlink:href",
                            format!("data:image/png;base64,{}", base64(&diff)),
                        ),
                );
            } else {
                doc = doc.add(
                    Image::new()
                        .set("x", w * 2 + 50)
                        .set("y", 0)
                        .set("width", w * 2)
                        .set("height", h * 2)
                        .set(
                            "xlink:href",
                            format!("data:image/png;base64,{}", base64(&png)),
                        ),
                );
            }
            let stroke = if show_blocks { "red" } else { "none" };
            for (id, block) in canvas.blocks.iter() {
                let mut cost = 0.0;
                for y in block.0 .1..block.1 .1 {
                    for x in block.0 .0..block.1 .0 {
                        cost += pixel_distance(
                            &png[y as usize][x as usize],
                            &canvas.bitmap[y as usize][x as usize],
                        );
                    }
                }
                cost = (cost * 0.005).round();
                let title = format!(
                    "block [{}]\n({}, {}) - ({}, {})\nw: {}, h: {}\ndiff = {}",
                    id, block.0 .0, block.0 .1, block.1 .0, block.1 .1, block.1.0 - block.0.0, block.1.1 - block.0.1, cost
                );
                doc = doc.add(
                    Group::new().add(Title::new().add(Text::new(&title))).add(
                        Rectangle::new()
                            .set("x", block.0 .0 * 2)
                            .set("y", 2 * h as i32 - block.1 .1 * 2)
                            .set("width", 2 * (block.1 .0 - block.0 .0))
                            .set("height", 2 * (block.1 .1 - block.0 .1))
                            .set("fill", "#0000")
                            .set("stroke-width", 2)
                            .set("stroke", stroke),
                    ),
                );
                doc = doc.add(
                    Group::new().add(Title::new().add(Text::new(&title))).add(
                        Rectangle::new()
                            .set("x", w as i32 * 2 + 50 + block.0 .0 * 2)
                            .set("y", 2 * h as i32 - block.1 .1 * 2)
                            .set("width", 2 * (block.1 .0 - block.0 .0))
                            .set("height", 2 * (block.1 .1 - block.0 .1))
                            .set("fill", "#0000")
                            .set("stroke-width", 2)
                            .set("stroke", stroke),
                    ),
                );
            }
        }
        Err(err) => {
            doc = doc.add(
                Image::new()
                    .set("x", w * 2 + 50)
                    .set("y", 0)
                    .set("width", w * 2)
                    .set("height", h * 2)
                    .set(
                        "xlink:href",
                        format!("data:image/png;base64,{}", base64(&png)),
                    ),
            );
            error = err.to_string()
        }
    }
    Ret {
        score: format!(
            "{} (?????????: {}, ?????????: {})",
            cost + similarity,
            cost,
            similarity
        ),
        error,
        svg: doc.to_string(),
    }
}

#[wasm_bindgen]
pub fn get_max_turn(output: String) -> i32 {
    console_error_panic_hook::set_once();
    read_isl(output.into_bytes().as_slice()).unwrap().len() as i32
}

#[wasm_bindgen]
pub fn get_reversed_program(output: String) -> String {
    console_error_panic_hook::set_once();
    match read_isl(output.clone().into_bytes().as_slice()) {
        Ok(program) => {
            let program = icfpc2022::wata::get_reversed_program(&program);
            let mut out = String::new();
            for p in program {
                let _ = writeln!(out, "{}", p);
            }
            out
        },
        Err(_) => {
            output
        }
    }
}
