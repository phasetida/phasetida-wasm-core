use std::collections::HashSet;

use wasm_bindgen::JsValue;

use crate::{
    LINE_STATES,
    chart::{self, JudgeLine, WithTimeRange},
    states::{LineState, Metadata, NoteState, get_seconds_per_tick},
};

pub fn init_line_states(chart: chart::Chart) -> Result<JsValue, JsValue> {
    LINE_STATES
        .try_with(|states_rc| {
            let mut states = states_rc.borrow_mut();
            let iter = chart.judge_line_list.into_iter().enumerate();
            let available_len = iter.len();
            iter.for_each(|(i, it)| {
                let JudgeLine {
                    bpm,
                    notes_above,
                    notes_below,
                    speed_events,
                    move_events,
                    rotate_events,
                    alpha_events,
                } = it;
                states[i] = LineState {
                    enable: true,
                    bpm,
                    move_events,
                    alpha_events,
                    speed_events,
                    rotate_events,
                    notes_above_state: notes_above
                        .clone()
                        .into_iter()
                        .map(|it| NoteState {
                            note: it,
                            ..std::default::Default::default()
                        })
                        .collect(),
                    notes_below_state: notes_below
                        .clone()
                        .into_iter()
                        .map(|it| NoteState {
                            note: it,
                            ..std::default::Default::default()
                        })
                        .collect(),
                    ..Default::default()
                }
            });
            (available_len..states.len()).for_each(|it| states[it].enable = false);
            process_highlight(states.as_mut());
            Ok(serde_wasm_bindgen::to_value(&get_metadata(states.as_ref()))
                .map_err(|e| format!("failed to serialize the metadata: {}", e))?)
        })
        .map_err(|_| "failed to access states")?
}

fn process_highlight(judge_line_states: &mut [LineState]) {
    let mut set1 = HashSet::<i32>::new();
    let mut set2 = HashSet::<i32>::new();
    judge_line_states.iter().for_each(|it| {
        if !it.enable {
            return;
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let mut process = |notes: &Vec<NoteState>| {
            notes.iter().for_each(|n| {
                let tick_time = n.note.time;
                let second_time = ((seconds_per_tick * 32768.0) as i32) * tick_time;
                if set1.contains(&second_time) {
                    set2.insert(second_time);
                } else {
                    set1.insert(second_time);
                }
            });
        };
        process(&it.notes_above_state);
        process(&it.notes_below_state);
    });
    judge_line_states.iter_mut().for_each(|it| {
        if !it.enable {
            return;
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let process = |notes: &mut Vec<NoteState>| {
            notes.iter_mut().for_each(|n| {
                let tick_time = n.note.time;
                let second_time = ((seconds_per_tick * 32768.0) as i32) * tick_time;
                if set2.contains(&second_time) {
                    n.highlight = true;
                }
            });
        };
        process(&mut it.notes_above_state);
        process(&mut it.notes_below_state);
    });
}

fn get_metadata(state: &[LineState]) -> Metadata {
    let note_max_time = state.iter().fold(0.0, |last, it| {
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        let get_time = |note: &NoteState| -> f64 {
            (note.note.time as f64 + note.note.hold_time) * seconds_per_tick
        };
        [
            it.notes_above_state.last().map(get_time).unwrap_or(0.0),
            it.notes_below_state.last().map(get_time).unwrap_or(0.0),
        ]
        .iter()
        .fold(last, |l, i| i.max(l))
    });
    let event_max_time = state.iter().fold(0.0, |last, it| {
        fn event_folder(seconds_per_tick: f64, events: &[impl WithTimeRange]) -> f64 {
            events
                .iter()
                .fold(0.0, |last, it| last.max(it.time_start() * seconds_per_tick))
        }
        let seconds_per_tick = get_seconds_per_tick(it.bpm);
        [
            event_folder(seconds_per_tick, &it.move_events),
            event_folder(seconds_per_tick, &it.alpha_events),
            event_folder(seconds_per_tick, &it.speed_events),
            event_folder(seconds_per_tick, &it.rotate_events),
        ]
        .iter()
        .fold(last, |l, i| i.max(l))
    });
    return Metadata {
        length_in_second: note_max_time.max(event_max_time),
    };
}
