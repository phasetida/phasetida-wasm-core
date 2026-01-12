use std::collections::HashSet;

use serde::Serialize;
use wasm_bindgen::JsValue;

use crate::{
    INPUT_BUFFER, LINE_STATES, TOUCH_STATES,
    chart::{self, JudgeLine, Note, NoteType, TimeState, WithTimeRange, WithValue},
    effect::{self},
    input::{self, TouchInfo},
    math::{self, Point},
};

pub struct LineState {
    pub enable: bool,
    pub x: f64,
    pub y: f64,
    pub rotate: f64,
    pub alpha: f64,
    pub speed: f64,
    pub line_y: f64,
    pub tick_time: f64,
    pub event_speed_index_cache: i32,
    pub event_move_index_cache: i32,
    pub event_rotate_index_cache: i32,
    pub event_alpha_index_cache: i32,
    pub notes_above_state: Vec<NoteState>,
    pub notes_below_state: Vec<NoteState>,
    pub speed_events: Vec<chart::Event1>,
    pub move_events: Vec<chart::Event4>,
    pub rotate_events: Vec<chart::Event2>,
    pub alpha_events: Vec<chart::Event2>,
    pub bpm: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteScore {
    Perfect,
    Good,
    Bad,
    Miss,
    None,
}

pub struct NoteState {
    pub note: chart::Note,
    pub highlight: bool,
    pub score: NoteScore,
    pub hold_cool_down: f64,
    pub extra_score: NoteScore,
}

#[derive(Serialize)]
struct Metadata {
    length_in_second: f64,
}

impl Default for LineState {
    fn default() -> Self {
        LineState {
            enable: false,
            x: 0.0,
            y: 0.0,
            rotate: 0.0,
            alpha: 0.0,
            speed: 1.0,
            line_y: 0.0,
            tick_time: 0.0,
            event_speed_index_cache: 0,
            event_move_index_cache: 0,
            event_rotate_index_cache: 0,
            event_alpha_index_cache: 0,
            notes_above_state: vec![],
            notes_below_state: vec![],
            speed_events: vec![],
            move_events: vec![],
            alpha_events: vec![],
            rotate_events: vec![],
            bpm: 0.0,
        }
    }
}

impl Default for NoteState {
    fn default() -> Self {
        NoteState {
            highlight: false,
            score: NoteScore::None,
            hold_cool_down: 0.0,
            extra_score: NoteScore::None,
            note: chart::Note {
                note_type: chart::NoteType::Tap,
                time: 0,
                position_x: 0.0,
                hold_time: 0.25,
                speed: 0.0,
                floor_position: 0.0,
            },
        }
    }
}

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

fn get_seconds_per_tick(bpm: f64) -> f64 {
    60.0 / bpm / 32.0
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

pub fn tick_lines(time_in_second: f64) -> Result<(), JsValue> {
    LINE_STATES
        .try_with(|it| {
            let mut x = it.borrow_mut();
            for state in x.iter_mut() {
                tick_line_state(time_in_second, state);
            }
        })
        .map_err(|_| "failed to access states")?;
    Ok(())
}

fn get_line_y(tick_time: f64, line: &LineState) -> f64 {
    let mut t = 0.0;
    let seconds_per_tick = 60.0 / line.bpm / 32.0;
    let speed_events = &line.speed_events;
    for event in speed_events {
        if event.end_time > tick_time && event.start_time > tick_time {
            break;
        }
        if event.start_time < tick_time && tick_time < event.end_time {
            let duration = event.end_time - event.start_time;
            let percent = (tick_time - event.start_time) / duration;
            t += duration * percent * event.value;
            break;
        }
        if event.end_time < tick_time {
            t += (event.end_time - event.start_time) * event.value
        }
    }
    t * seconds_per_tick
}

fn tick_line_state(time_in_second: f64, state: &mut LineState) {
    let seconds_per_tick = 60.0 / state.bpm / 32.0;
    let tick_time = time_in_second / seconds_per_tick;
    let ((speed_value, _), _, speed_new_index) = get_current_value_for_event(
        tick_time,
        &state.speed_events,
        state.event_speed_index_cache,
    );
    state.event_speed_index_cache = speed_new_index;
    state.speed = speed_value;
    let ((alpha_start, alpha_end), alpha_percent, alpha_new_index) = get_current_value_for_event(
        tick_time,
        &state.alpha_events,
        state.event_alpha_index_cache,
    );
    state.alpha = alpha_start + (alpha_end - alpha_start) * alpha_percent;
    state.event_alpha_index_cache = alpha_new_index;
    let ((rotate_start, rotate_end), rotate_percent, rotate_new_index) =
        get_current_value_for_event(
            tick_time,
            &state.rotate_events,
            state.event_rotate_index_cache,
        );
    state.rotate =
        math::fix_degree(360.0 - (rotate_start + (rotate_end - rotate_start) * rotate_percent));
    state.event_rotate_index_cache = rotate_new_index;
    let (((line_x_start, line_x_end), (line_y_start, line_y_end)), line_percent, line_new_index) =
        get_current_value_for_event(tick_time, &state.move_events, state.event_move_index_cache);
    state.x = math::WORLD_WIDTH * (line_x_start + (line_x_end - line_x_start) * line_percent);
    state.y =
        math::WORLD_HEIGHT * (1.0 - (line_y_start + (line_y_end - line_y_start) * line_percent));
    state.event_move_index_cache = line_new_index;
    state.line_y = get_line_y(tick_time, state);
    state.tick_time = tick_time;
}

fn get_current_value_for_event<T, U>(
    tick_time: f64,
    events: &[U],
    cache_index: i32,
) -> ((T, T), f64, i32)
where
    U: WithValue<T> + WithTimeRange,
{
    if events.is_empty() {
        return (U::zero(), 0.0, 0);
    }
    let event_result = find_current_event(tick_time, events, cache_index);
    match event_result {
        Ok((event, index, percent)) => (event.get_value(), percent, index),
        Err(_) => (U::zero(), 0.0, 0),
    }
}

fn find_current_event<T>(
    tick_time: f64,
    events: &[T],
    cache_index: i32,
) -> Result<(&T, i32, f64), ()>
where
    T: WithTimeRange,
{
    let mut i = cache_index.clamp(0, events.len() as i32);
    let mut last_result: TimeState = TimeState::During(0.0);
    loop {
        let op = events.get(i as usize);
        match op {
            Some(event) => {
                let result = event.check_time(tick_time);
                match result {
                    chart::TimeState::Early => {
                        if last_result == chart::TimeState::Late {
                            return Ok((event, i, 1.0));
                        }
                        i -= 1
                    }
                    chart::TimeState::Late => {
                        if last_result == chart::TimeState::Early {
                            return Ok((event, i, 1.0));
                        }
                        i += 1
                    }
                    chart::TimeState::During(percent) => return Ok((event, i, percent)),
                }
                last_result = result;
            }
            None => {
                if i <= 0 {
                    return Err(());
                }
                return match events.last() {
                    Some(x) => Ok((x, events.len() as i32 - 1, 1.0)),
                    None => Err(()),
                };
            }
        }
    }
}

pub fn tick_lines_judge(delta_time_in_second: f64, auto: bool) -> Result<(), JsValue> {
    INPUT_BUFFER.with(input::process_touch_info)?;
    TOUCH_STATES
        .try_with(|touches_ref| {
            let mut touches = touches_ref.borrow_mut();
            LINE_STATES
                .try_with(|state_ref| {
                    let mut lines = state_ref.borrow_mut();
                    tick_line_judge(delta_time_in_second, touches.as_mut(), lines.as_mut(), auto);
                })
                .map_err(|_| "failed to access state")
        })
        .map_err(|_| "failed to access state")??;
    Ok(())
}

fn tick_line_judge(
    delta_time_in_second: f64,
    touches: &mut [TouchInfo],
    lines: &mut [LineState],
    auto: bool,
) {
    lines.iter_mut().for_each(|line| {
        if !line.enable {
            return;
        }
        let current_tick = line.tick_time;
        line.notes_above_state
            .iter_mut()
            .chain(line.notes_below_state.iter_mut())
            .for_each(|note| {
                let line_x = line.x;
                let line_y = line.y;
                let line_rotate = line.rotate;
                let bpm = line.bpm;
                let note_type = note.note.note_type;
                if auto {
                    match note_type {
                        NoteType::Hold => tick_hold_note_auto(
                            delta_time_in_second,
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        _ => tick_normal_note_auto(
                            current_tick,
                            note,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                    }
                } else {
                    match note_type {
                        NoteType::Tap => tick_tap_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Drag => tick_drag_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Hold => tick_hold_note(
                            delta_time_in_second,
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                        NoteType::Flick => tick_flick_note(
                            current_tick,
                            note,
                            touches,
                            line_x,
                            line_y,
                            line_rotate,
                            bpm,
                        ),
                    }
                }
            });
    });
    touches.iter_mut().for_each(|touch| {
        if touch.enable {
            touch.touch_valid = false;
        }
    });
}

fn check_point_in_judge_range(
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    Note {
        position_x: note_position_x,
        ..
    }: &Note,
    TouchInfo {
        x: touch_x,
        y: touch_y,
        ..
    }: &TouchInfo,
) -> (bool, (f64, f64)) {
    let Point {
        x: root_x,
        y: root_y,
    } = math::get_pos_out_of_line(
        line_x,
        line_y,
        line_rotate,
        *note_position_x * math::UNIT_WIDTH,
    );
    let Point {
        x: touch_root_x,
        y: touch_root_y,
    } = math::get_pos_point_vertical_in_line(
        line_x,
        line_y,
        line_rotate,
        *touch_x as f64,
        *touch_y as f64,
    );
    (
        math::is_point_in_judge_range(
            root_x,
            root_y,
            math::fix_degree(line_rotate),
            touch_root_x,
            touch_root_y,
            300.0,
        ),
        (root_x, root_y),
    )
}

fn check_judge_result(current_tick: f64, note: &NoteState, bpm: f64) -> (f64, NoteScore) {
    let seconds_per_tick = 60.0 / bpm / 32.0;
    let perfect_range_in_tick = 0.08 / seconds_per_tick;
    let good_range_in_tick = 0.16 / seconds_per_tick;
    let bad_range_in_tick = 0.18 / seconds_per_tick;
    let time_delta = current_tick - note.note.time as f64;
    (
        time_delta,
        match time_delta.abs() {
            x if 0.0 <= x && x <= perfect_range_in_tick => NoteScore::Perfect,
            x if perfect_range_in_tick < x && x <= good_range_in_tick => NoteScore::Good,
            x if good_range_in_tick < x && x <= bad_range_in_tick => NoteScore::Bad,
            _ => NoteScore::Miss,
        },
    )
}

fn create_splash(x: f64, y: f64, note_score: NoteScore) {
    match note_score {
        NoteScore::Perfect => effect::new_effect(x, y, 0),
        NoteScore::Good => effect::new_effect(x, y, 1),
        _ => {}
    };
}

fn tick_normal_note_auto(
    current_tick: f64,
    note: &mut NoteState,
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    let (judge_delta, _) = check_judge_result(current_tick, note, bpm);
    if judge_delta >= 0.0 {
        let Point {
            x: root_x,
            y: root_y,
        } = math::get_pos_out_of_line(
            line_x,
            line_y,
            line_rotate,
            note.note.position_x * math::UNIT_WIDTH,
        );
        note.score = NoteScore::Perfect;
        create_splash(root_x, root_y, NoteScore::Perfect);
    }
}

fn tick_flick_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return;
    }
    if note.extra_score != NoteScore::None {
        if judge_delta > 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            note.score = NoteScore::Perfect;
            create_splash(root_x, root_y, NoteScore::Perfect);
        }
        return;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.length() >= 50.0 {
            note.extra_score = NoteScore::Perfect;
            touch.reset_length();
            return;
        }
    }
}

fn tick_hold_note_auto(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    let (judge_delta, _) = check_judge_result(current_tick, note, bpm);
    if judge_delta >= 0.0 {
        note.extra_score = NoteScore::Perfect;
    }
    tick_hold_note_common(
        delta_time_in_second,
        current_tick,
        note,
        touches,
        line_x,
        line_y,
        line_rotate,
        bpm,
        true,
    );
}

fn tick_hold_note_common(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
    auto: bool,
) -> bool {
    if note.extra_score != NoteScore::None {
        let seconds_per_tick = 60.0 / bpm / 32.0;
        let delta_tick = delta_time_in_second / seconds_per_tick;
        note.hold_cool_down -= delta_tick;
        if note.hold_cool_down <= 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            if auto
                || touches.iter().any(|touch| {
                    let (is_in_judge_range, _) =
                        check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
                    is_in_judge_range && touch.enable
                })
            {
                note.hold_cool_down += 16.0;
                create_splash(root_x, root_y, note.extra_score);
            } else {
                note.score = NoteScore::Miss;
            }
        }
        if note.note.hold_time + note.note.time as f64 <= current_tick {
            note.score = note.extra_score;
        }
        return true;
    }
    false
}

fn tick_hold_note(
    delta_time_in_second: f64,
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    if tick_hold_note_common(
        delta_time_in_second,
        current_tick,
        note,
        touches,
        line_x,
        line_y,
        line_rotate,
        bpm,
        false,
    ) {
        return;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.touch_valid {
            if judge_result != NoteScore::Perfect && judge_result != NoteScore::Good {
                continue;
            }
            touch.touch_valid = false;
            note.extra_score = judge_result;
            return;
        }
    }
}

fn tick_drag_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return;
    }
    if note.extra_score != NoteScore::None {
        if judge_delta > 0.0 {
            let Point {
                x: root_x,
                y: root_y,
            } = math::get_pos_out_of_line(
                line_x,
                line_y,
                line_rotate,
                note.note.position_x * math::UNIT_WIDTH,
            );
            note.score = NoteScore::Perfect;
            create_splash(root_x, root_y, NoteScore::Perfect);
        }
        return;
    }
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, _) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range {
            note.extra_score = NoteScore::Perfect;
            return;
        }
    }
}

fn tick_tap_note(
    current_tick: f64,
    note: &mut NoteState,
    touches: &mut [TouchInfo],
    line_x: f64,
    line_y: f64,
    line_rotate: f64,
    bpm: f64,
) {
    if note.score != NoteScore::None {
        return;
    }
    let (judge_delta, judge_result) = check_judge_result(current_tick, note, bpm);
    if judge_delta < 0.0 && judge_result == NoteScore::Miss {
        return;
    }
    //+ late
    if judge_delta > 0.0 && judge_result == NoteScore::Miss {
        note.score = NoteScore::Miss;
        return;
    }
    for touch in touches {
        if !touch.enable {
            continue;
        }
        let (is_in_judge_range, (root_x, root_y)) =
            check_point_in_judge_range(line_x, line_y, line_rotate, &note.note, touch);
        if is_in_judge_range && touch.touch_valid {
            touch.touch_valid = false;
            note.score = judge_result;
            create_splash(root_x, root_y, judge_result);
            return;
        }
    }
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

pub fn reset_note_state(before_time_in_second: f64) -> Result<(), JsValue> {
    LINE_STATES
        .try_with(|states_rc| {
            let mut state = states_rc.borrow_mut();
            state.iter_mut().for_each(|line| {
                let seconds_per_tick = get_seconds_per_tick(line.bpm);
                let process_notes = |notes: &mut [NoteState]| {
                    notes.iter_mut().for_each(|note| {
                        note.hold_cool_down = 0.0;
                        let note_time_in_second = note.note.time as f64 * seconds_per_tick;
                        let hold_time_in_second =
                            (note.note.time as f64 + note.note.hold_time) * seconds_per_tick;
                        if note_time_in_second >= before_time_in_second {
                            note.extra_score = NoteScore::None;
                            note.score = NoteScore::None;
                        } else if hold_time_in_second >= before_time_in_second {
                            note.score = NoteScore::None;
                        }
                    });
                };
                process_notes(&mut line.notes_above_state);
                process_notes(&mut line.notes_below_state);
            });
        })
        .map_err(|_| "failed to access states".into())
}
