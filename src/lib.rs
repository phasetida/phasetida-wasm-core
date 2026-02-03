mod buffer_wasm;
mod input_wasm;
mod renders_wasm;

use phasetida_core::{Chart, init_line_states, process_state_to_drawable, tick_all};
use wasm_bindgen::prelude::*;

use crate::buffer_wasm::Uint8ArrayWrapper;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_namespace=window , js_name = outputBuffer)]
    static OUTPUT_BUFFER: js_sys::Uint8Array;

    #[wasm_bindgen(thread_local_v2,  js_namespace=window,js_name = outputBufferLength)]
    static OUTPUT_BUFFER_LENGTH: usize;

    #[wasm_bindgen(thread_local_v2, js_namespace=window , js_name = inputBuffer)]
    static INPUT_BUFFER: js_sys::Uint8Array;

    #[wasm_bindgen(thread_local_v2,  js_namespace=window,js_name = inputBufferLength)]
    static INPUT_BUFFER_LENGTH: usize;
}

#[wasm_bindgen]
pub fn load_image_offset(
    hold_head_height: f64,
    hold_head_highlight_height: f64,
    hold_end_height: f64,
    hold_end_highlight_height: f64,
) {
    phasetida_core::load_image_offset(
        hold_head_height,
        hold_head_highlight_height,
        hold_end_height,
        hold_end_highlight_height,
    );
}

#[wasm_bindgen]
pub fn load_level(chart_json: &str) -> Result<JsValue, JsValue> {
    let result: Chart =
        serde_json::from_str(chart_json).map_err(|e| format!("failed to analyze, {}", e))?;
    let meta = init_line_states(result);
    Ok(serde_wasm_bindgen::to_value(&meta)
        .map_err(|e| format!("failed to serialize the metadata: {}", e))?)
}

#[wasm_bindgen]
pub fn pre_draw(time_in_second: f64, delta_time_in_second: f64, auto: bool) {
    INPUT_BUFFER.with(input_wasm::process_touch_info);
    tick_all(time_in_second, delta_time_in_second, auto);
    OUTPUT_BUFFER.with(|buf| {
        process_state_to_drawable(&mut Uint8ArrayWrapper {
            buffer: buf,
            cursor: 0,
        })
    });
}

#[wasm_bindgen]
pub fn reset_note_state(before_time_in_second: f64) {
    phasetida_core::reset_note_state(before_time_in_second);
}
