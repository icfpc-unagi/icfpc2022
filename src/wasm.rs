// NOTE: To enable completion, add vscode settings "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
use crate::{color::geometric_median_4d, *};
use svg::node::{
    element::{Group, Image, Rectangle, Title},
    Text,
};
use wasm_bindgen::{prelude::*, *};
use web_sys::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvases = document.query_selector_all("canvas[isl]")?;
    for i in 0..canvases.length() {
        let el_canvas: HtmlCanvasElement = JsValue::from(canvases.get(i)).into();
        let isl = el_canvas.get_attribute("isl").unwrap();
        let mut managed = ManagedCanvas::new(&el_canvas, &[], "", &[]);
        managed.apply(&isl)?;
    }

    console::log_1(&JsValue::from("wasm initialized!"));
    Ok(())
}

#[wasm_bindgen]
pub struct ManagedCanvas {
    ctx: CanvasRenderingContext2d,
    model: Canvas,
    target: Vec<Vec<Color>>,
    cost: f64,
    isl: Vec<Move>,
}

#[wasm_bindgen]
impl ManagedCanvas {
    #[wasm_bindgen(constructor)]
    pub fn new(
        el_canvas: &HtmlCanvasElement,
        target_png: &[u8],
        init_config: &str, // len() == 0 if absent
        init_png: &[u8],   // len() == 0 if absent
    ) -> Self {
        Self {
            ctx: JsValue::from(el_canvas.get_context("2d").unwrap()).into(),
            // TODO: use init_config as InitialJson
            model: if init_png.len() > 0 {
                Canvas::from(read_png_r(init_png))
            } else {
                Canvas::new400()
            },
            target: read_png_r(target_png),
            cost: Default::default(),
            isl: Default::default(),
        }
    }

    #[wasm_bindgen(readonly)]
    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn render(&self) -> Result<(), JsValue> {
        self.ctx.clear_rect(0.0, 0.0, 0.0, 0.0);
        render_bitmap(&self.ctx, &self.model.bitmap)
    }

    pub fn apply(&mut self, isl: &str) -> Result<f64, JsValue> {
        let isl = read_isl(isl.as_bytes()).map_err(|e| JsValue::from(e.to_string()))?;
        for mov in &isl {
            match self.model.apply_safe(&mov) {
                Ok(cost) => {
                    self.cost += cost;
                    self.isl.push(mov.clone());
                }
                Err(e) => return Err(JsValue::from_str(&e.to_string())),
            }
        }
        self.render()?;
        Ok(self.cost)
    }

    pub fn clear(&mut self) -> Result<f64, JsValue> {
        self.model = Canvas::new400();
        self.cost = Default::default();
        self.isl.clear();
        self.render()?;
        Ok(self.cost)
    }

    pub fn geometric_median_4d_on_rect(
        &self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) -> Result<Vec<f64>, JsValue> {
        if x0 >= x1 || y0 >= y1 {
            return Err(JsValue::from_str(&format!(
                "invalid range {x0},{y0},{x1},{y1}"
            )));
        }
        let points: Vec<_> = (y0..y1)
            .flat_map(|y| {
                (x0..x1).map(move |x| self.model.bitmap[y as usize][x as usize].map(|c| c as f64))
            })
            .collect();
        Ok(geometric_median_4d(&points)
            .map(|c| c.round().clamp(0.0, 255.0))
            .into())
    }

    pub fn svg(&self) -> Result<String, JsValue> {
        // とりあえず通るように
        let w = 400;
        let h = 400;
        let program = &self.isl;
        let t = program.len();
        let canvas = &self.model;
        let show_blocks = true;
        let png = &self.target;
        // let cost = self.cost;
        // web/src/lib.rs から丸コピ, png なし, 座標を x1 scale に, init もなし
        let mut doc = svg::Document::new()
            .set("viewBox", (0, 0, w, h))
            .set("width", w)
            .set("height", h);
        let mut dummy_canvas = Canvas::new(w, h);
        let _ = dummy_canvas.apply_all_safe(program[0..t as usize].iter().cloned());
        // let mut canvas = if let Some(json) = INIT_CANVAS[problem_id] {
        //     let mut json: InitialJson = serde_json::from_reader(json).unwrap();
        //     if problem_id >= 35 {
        //         json.source_png_p_n_g = None;
        //         json.blocks[0].color = Some([0, 0, 0, 0]);
        //     }
        //     let mut canvas = Canvas::try_from(json).unwrap();
        //     if problem_id >= 35 {
        //         canvas.bitmap = read_png(&INIT_PNG[problem_id]);
        //         canvas.cost_type = CostType::V2;
        //     }
        //     canvas
        // } else {
        //     Canvas::new(png[0].len(), png.len())
        // };
        // if swap_input {
        //     canvas.bitmap = png.clone();
        // }
        let png = wata::get_swapped_png(png, &program[t as usize..], &dummy_canvas);
        // match canvas.apply_all_safe(program[0..t as usize].iter().cloned()) {
        //     Ok(s) => cost = s.round() as i64,
        //     Err(e) => {
        //         error = e.to_string();
        //     }
        // }
        // similarity = icfpc2022::similarity(&png, &canvas.bitmap).round() as i64;
        // doc = doc.add(
        //     Image::new()
        //         .set("x", 0)
        //         .set("y", 0)
        //         .set("width", w)
        //         .set("height", h)
        //         .set(
        //             "xlink:href",
        //             format!("data:image/png;base64,{}", base64(&canvas.bitmap)),
        //         ),
        // );
        // if show_diff {
        //     let diff = pixel_distance_bitmap(&png, &canvas.bitmap);
        //     doc = doc.add(
        //         Image::new()
        //             .set("x", 0)
        //             .set("y", 0)
        //             .set("width", w)
        //             .set("height", h)
        //             .set(
        //                 "xlink:href",
        //                 format!("data:image/png;base64,{}", base64(&diff)),
        //             ),
        //     );
        // } else {
        doc = doc.add(
            Image::new()
                .set("x", 0)
                .set("y", 0)
                .set("width", w)
                .set("height", h)
                .set("style", "opacity:0.5")
                .set(
                    "xlink:href",
                    format!("data:image/png;base64,{}", base64(&png)),
                ),
        );
        // }
        let stroke = if show_blocks { "red" } else { "none" };
        for (id, block) in canvas.blocks.iter() {
            // let mut cost = 0.0;
            // for y in block.0 .1..block.1 .1 {
            //     for x in block.0 .0..block.1 .0 {
            //         cost += pixel_distance(
            //             &png[y as usize][x as usize],
            //             &canvas.bitmap[y as usize][x as usize],
            //         );
            //     }
            // }
            // cost = (cost * 0.005).round();
            let title = format!(
                "block [{}]\n({}, {}) - ({}, {})",
                id,
                block.0 .0,
                block.0 .1,
                block.1 .0,
                block.1 .1,
                //cost
            );
            doc = doc.add(
                Group::new().add(Title::new().add(Text::new(&title))).add(
                    Rectangle::new()
                        .set("x", block.0 .0)
                        .set("y", h as i32 - block.1 .1)
                        .set("width", block.1 .0 - block.0 .0)
                        .set("height", block.1 .1 - block.0 .1)
                        .set("fill", "#0000")
                        .set("stroke-width", 2)
                        .set("stroke", stroke)
                        .set("block-id", id.to_string()),
                ),
            );
            // doc = doc.add(
            //     Group::new().add(Title::new().add(Text::new(&title))).add(
            //         Rectangle::new()
            //             .set("x", w as i32 * 2 + 50 + block.0 .0 * 2)
            //             .set("y", 2 * h as i32 - block.1 .1 * 2)
            //             .set("width", 2 * (block.1 .0 - block.0 .0))
            //             .set("height", 2 * (block.1 .1 - block.0 .1))
            //             .set("fill", "#0000")
            //             .set("stroke-width", 2)
            //             .set("stroke", stroke),
            //     ),
            // );
        }
        Ok(doc.to_string())
    }
}

#[wasm_bindgen]
pub fn run_isl_all(ctx: &CanvasRenderingContext2d, isl: &str) -> Result<(), JsValue> {
    let isl = read_isl(isl.as_bytes()).unwrap();
    let mut canvas = Canvas::new400();
    canvas.apply_all(isl);
    render_bitmap(ctx, &canvas.bitmap)
}

fn render_bitmap(ctx: &CanvasRenderingContext2d, bitmap: &Vec<Vec<Color>>) -> Result<(), JsValue> {
    let v: Vec<_> = bitmap.iter().rev().flatten().flatten().cloned().collect();
    let imagedata = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&v),
        bitmap[0].len() as u32,
        bitmap.len() as u32,
    )?;
    ctx.put_image_data(&imagedata, 0.0, 0.0)
}

// from web/src/lib.rs
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
