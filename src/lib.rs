mod chart;
mod draw;
mod effect;
mod input;
mod math;
mod renders;
mod states;
mod states_initializing;
mod states_judge;
mod states_statistics;
mod states_ticking;

use crate::draw::process_state_to_drawable;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

thread_local! {
    pub static FLATTEN_NOTE_INDEX:Rc<RefCell<Vec<states_statistics::NoteIndex>>>= Rc::new(RefCell::new(Vec::<_>::new()));
    pub static LINE_STATES: Rc<RefCell<[states::LineState;50]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
    pub static TOUCH_STATES: Rc<RefCell<[input::TouchInfo; 30]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
    pub static HIT_EFFECT_POOL: Rc<RefCell<[effect::HitEffect; 64]>> = Rc::new(RefCell::new(std::array::from_fn(|_|std::default::Default::default())));
    pub static CHART_STATISTICS: Rc<RefCell<states_statistics::ChartStatistics>>= Rc::new(RefCell::new(std::default::Default::default()));
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
pub fn load_level(js_value: &str) -> Result<JsValue, JsValue> {
    let mut result: chart::Chart =
        serde_json::from_str(js_value).map_err(|e| format!("failed to analyze, {}", e))?;
    result.judge_line_list = result
        .judge_line_list
        .into_iter()
        .map(|mut line| {
            line.notes_above.sort_by(|a, b| (a.time).cmp(&b.time));
            line.notes_below.sort_by(|a, b| (a.time).cmp(&b.time));
            line
        })
        .collect::<Vec<_>>();
    let meta = states_initializing::init_line_states(result)?;
    states_statistics::init_flatten_line_state()?;
    Ok(meta)
}

#[wasm_bindgen]
pub fn pre_draw(time_in_second: f64, delta_time_in_second: f64, auto: bool) -> Result<(), JsValue> {
    states_ticking::tick_lines(time_in_second)?;
    effect::tick_effect(delta_time_in_second)?;
    let judged = states_judge::tick_lines_judge(delta_time_in_second, auto)?;
    if judged {
        states_statistics::refresh_chart_statistics()?;
    }
    OUTPUT_BUFFER.with(|buf| process_state_to_drawable(buf))?;
    Ok(())
}

#[wasm_bindgen]
pub fn reset_note_state(before_time_in_second: f64) -> Result<(), JsValue> {
    states::reset_note_state(before_time_in_second)
}
