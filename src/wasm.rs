// NOTE: To enable completion, add vscode settings "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
use crate::*;
use wasm_bindgen::{prelude::*, *};
use web_sys::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvases = document.query_selector_all("canvas[isl]")?;
    for i in 0..canvases.length() {
        let canvas = JsValue::from(canvases.get(i));
        let isl = Element::from(canvas.clone()).get_attribute("isl").unwrap();
        let mut managed = ManagedCanvas::new(canvas.into());
        managed.apply(&isl)?;
        managed.render()?;
    }
    console::log_1(&JsValue::from("wasm initialized!!"));
    Ok(())
}

#[wasm_bindgen]
pub struct ManagedCanvas {
    ctx: CanvasRenderingContext2d,
    model: Canvas,
}

#[wasm_bindgen]
impl ManagedCanvas {
    #[wasm_bindgen(constructor)]
    pub fn new(el_canvas: HtmlCanvasElement) -> Self {
        Self {
            ctx: JsValue::from(el_canvas.get_context("2d").unwrap()).into(),
            model: Canvas::new400(),
        }
    }

    pub fn render(&self) -> Result<(), JsValue> {
        self.ctx.clear_rect(0.0, 0.0, 0.0, 0.0);
        render_bitmap(&self.ctx, &self.model.bitmap)
    }

    pub fn apply(&mut self, isl: &str) -> Result<(), JsValue> {
        let isl = read_isl(isl.as_bytes()).unwrap();
        self.model.apply_all(isl);
        self.render()
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
