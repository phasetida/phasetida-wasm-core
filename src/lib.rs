mod buffer_wasm;
mod input_wasm;
mod renders_wasm;

use phasetida_core::{
    chart, draw, effect, states, states_initializing, states_judge, states_statistics,
    states_ticking,
};
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
    draw::load_image_offset(
        hold_head_height,
        hold_head_highlight_height,
        hold_end_height,
        hold_end_highlight_height,
    );
}

#[wasm_bindgen]
pub fn load_level(chart_json: &str) -> Result<JsValue, JsValue> {
    let mut result: chart::Chart =
        serde_json::from_str(chart_json).map_err(|e| format!("failed to analyze, {}", e))?;
    result.judge_line_list = result
        .judge_line_list
        .into_iter()
        .map(|mut line| {
            line.notes_above.sort_by(|a, b| (a.time).cmp(&b.time));
            line.notes_below.sort_by(|a, b| (a.time).cmp(&b.time));
            line
        })
        .collect::<Vec<_>>();
    let meta = states_initializing::init_line_states(result);
    states_statistics::init_flatten_line_state();
    Ok(serde_wasm_bindgen::to_value(&meta)
        .map_err(|e| format!("failed to serialize the metadata: {}", e))?)
}

#[wasm_bindgen]
pub fn pre_draw(time_in_second: f64, delta_time_in_second: f64, auto: bool) {
    states_ticking::tick_lines(time_in_second);
    effect::tick_effect(delta_time_in_second);
    INPUT_BUFFER.with(input_wasm::process_touch_info);
    let judged = states_judge::tick_lines_judge(delta_time_in_second, auto);
    if judged {
        states_statistics::refresh_chart_statistics();
    }
    OUTPUT_BUFFER.with(|buf| {
        draw::process_state_to_drawable(&mut Uint8ArrayWrapper {
            buffer: buf,
            cursor: 0,
        })
    });
}

#[wasm_bindgen]
pub fn reset_note_state(before_time_in_second: f64) {
    states::reset_note_state(before_time_in_second);
    states_statistics::refresh_chart_statistics();
}
