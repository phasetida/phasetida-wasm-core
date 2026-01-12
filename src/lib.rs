mod chart;
mod draw;
mod effect;
mod input;
mod math;
mod renders;
mod states;

use crate::draw::process_state_to_drawable;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

thread_local! {
    pub static LINE_STATES: Rc<RefCell<[states::LineState;50]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
    pub static TOUCH_STATES: Rc<RefCell<[input::TouchInfo; 30]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
    pub static HIT_EFFECT_POOL: Rc<RefCell<[effect::HitEffect; 64]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
}

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
pub fn load_level(js_value: &str) -> Result<bool, JsValue> {
    let mut result: chart::Chart =
        serde_json::from_str(js_value).map_err(|e| format!("failed to analyze, {}", e))?;
    let result_lines = result
        .judge_line_list
        .into_iter()
        .map(|mut line| {
            line.notes_above.sort_by(|a, b| (a.time).cmp(&b.time));
            line.notes_below.sort_by(|a, b| (a.time).cmp(&b.time));
            line
        })
        .collect::<Vec<_>>();
    result.judge_line_list = result_lines;
    states::init_line_states(result).map_err(|e| format!("failed to initialize: {:?}", e))?;
    Ok(true)
}

#[wasm_bindgen]
pub fn pre_draw(
    time_in_second: f64,
    delta_time_in_second: f64,
    simultaneous_highlight: bool,
    auto: bool,
) -> Result<(), JsValue> {
    states::tick_lines(time_in_second)?;
    effect::tick_effect(delta_time_in_second)?;
    states::tick_lines_judge(delta_time_in_second, auto)?;
    OUTPUT_BUFFER.with(|buf| process_state_to_drawable(buf, simultaneous_highlight))?;
    Ok(())
}
