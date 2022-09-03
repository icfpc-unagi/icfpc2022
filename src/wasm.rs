// NOTE: To enable completion, add vscode settings "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
use crate::*;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, ImageData};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen]
pub fn run_isl_all(ctx: &CanvasRenderingContext2d, isl: &str) -> Result<(), JsValue> {
    let isl = read_isl(isl.as_bytes()).unwrap();
    let mut canvas = Canvas::new400();
    canvas.apply_all(isl);
    render_bitmap(ctx, &canvas.bitmap)
}

fn render_bitmap(ctx: &CanvasRenderingContext2d, bitmap: &Vec<Vec<Color>>) -> Result<(), JsValue> {
    let v: Vec<_> = bitmap.iter().flatten().flatten().cloned().collect();
    let imagedata = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&v),
        bitmap[0].len() as u32,
        bitmap.len() as u32,
    )?;
    ctx.put_image_data(&imagedata, 0.0, 0.0)
}
