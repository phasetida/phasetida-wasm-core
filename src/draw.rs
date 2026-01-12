use wasm_bindgen::prelude::*;

use crate::chart::{Note, NoteType};
use crate::effect::HitEffect;
use crate::math::{self, Point};
use crate::renders::{self, Dense, RendEffect, RendNote, RendPoint};
use crate::states::{LineState, NoteScore, NoteState};
use crate::{HIT_EFFECT_POOL, LINE_STATES, TOUCH_STATES};

struct BufferWithCursor<'a> {
    buffer: &'a js_sys::Uint8Array,
    cursor: usize,
}

impl<'a> BufferWithCursor<'a> {
    fn write(&mut self, slice: &[u8]) {
        slice.iter().for_each(|it| {
            self.buffer.set_index(self.cursor as u32, *it);
            self.cursor += 1;
        });
    }
}

pub fn process_state_to_drawable(
    output_buffer: &js_sys::Uint8Array,
    simultaneous_highlight: bool,
) -> Result<(), JsValue> {
    let mut wrapped_buffer = BufferWithCursor {
        buffer: output_buffer,
        cursor: 0,
    };
    LINE_STATES
        .try_with(|states_ref| {
            let states = states_ref.borrow();
            states
                .iter()
                .for_each(|it| write_line(&mut wrapped_buffer, it));
            write_notes(&mut wrapped_buffer, states.as_ref(), simultaneous_highlight);
        })
        .map_err(|_| "failed to access states")?;
    HIT_EFFECT_POOL
        .try_with(|effects_ref| {
            let effects = effects_ref.borrow();
            write_effects(&mut wrapped_buffer, effects.as_ref());
        })
        .map_err(|_| "failed to access states")?;
    TOUCH_STATES
        .try_with(|touches_ref| {
            let touches = touches_ref.borrow();
            touches.iter().for_each(|it| {
                if !it.enable {
                    return;
                }
                wrapped_buffer.write(
                    RendPoint {
                        rend_type: 4,
                        x: it.x,
                        y: it.y,
                    }
                    .to_bytes(),
                );
            });
        })
        .map_err(|_| "failed to access states")?;
    wrapped_buffer.write(&[0]);
    Ok(())
}

fn write_effects(wrapped_buffer: &mut BufferWithCursor, states: &[HitEffect]) {
    states.iter().for_each(|it| {
        if !it.enable {
            return;
        }
        wrapped_buffer.write(
            RendEffect {
                rend_type: 3,
                x: it.x as f32,
                y: it.y as f32,
                frame: ((30.0 * it.progress).floor() as i8).clamp(0, 29),
                tint_type: it.tint_type,
            }
            .to_bytes(),
        );
    });
}

fn write_line(wrapped_buffer: &mut BufferWithCursor, state: &LineState) {
    let p1 = math::get_cross_point_with_screen(state.x, state.y, math::fix_degree(state.rotate));
    let p2 =
        math::get_cross_point_with_screen(state.x, state.y, math::fix_degree(state.rotate + 180.0));
    let line = renders::RendLine {
        rend_type: 1,
        x1: p1.x as f32,
        y1: p1.y as f32,
        x2: p2.x as f32,
        y2: p2.y as f32,
        alpha: state.alpha as f32,
    };
    let line_slice = line.to_bytes();
    wrapped_buffer.write(line_slice);
}

fn write_notes(
    wrapped_buffer: &mut BufferWithCursor,
    states: &[LineState],
    simultaneous_highlight: bool,
) {
    let notes = states
        .iter()
        .map(|it| process_notes(it, simultaneous_highlight))
        .collect::<Vec<_>>();
    notes
        .iter()
        .for_each(|(_, it)| it.iter().for_each(|it| wrapped_buffer.write(it.to_bytes())));
    notes
        .iter()
        .for_each(|(it, _)| it.iter().for_each(|it| wrapped_buffer.write(it.to_bytes())));
}

fn process_notes(
    state: &LineState,
    simultaneous_highlight: bool,
) -> (Vec<RendNote>, Vec<RendNote>) {
    let mut vec = Vec::<RendNote>::new();
    let mut hold_vec = Vec::<RendNote>::new();
    let line_y = state.line_y;
    fn in_bound(x: f64, y: f64) -> bool {
        (-200.0..=2120.0).contains(&x) && (-200.0..=1280.0).contains(&y)
    }
    let seconds_per_tick = 60.0 / state.bpm / 32.0;
    let process = |notes: &Vec<NoteState>,
                   reverse: bool,
                   out: &mut Vec<RendNote>,
                   out_hold: &mut Vec<RendNote>| {
        let LineState { x, y, rotate, .. } = state;
        let iter = notes.iter();
        for note_state in iter {
            let NoteState {
                note:
                    Note {
                        time,
                        note_type,
                        position_x,
                        floor_position,
                        speed,
                        hold_time,
                    },
                highlight,
                score,
                ..
            } = note_state;
            if *score != NoteScore::None && *note_type != NoteType::Hold {
                continue;
            }
            let should_high_light: i8 = if *highlight && simultaneous_highlight {
                1
            } else {
                0
            };
            match note_type {
                NoteType::Tap | NoteType::Drag | NoteType::Flick => {
                    let delta_y = floor_position - line_y;
                    if *time <= state.tick_time as i32 || line_y > *floor_position + 0.001 {
                        continue;
                    }
                    let Point { x: raw_x, y: raw_y } =
                        math::get_pos_out_of_line(*x, *y, *rotate, position_x * math::UNIT_WIDTH);
                    let Point { x, y } = math::get_pos_out_of_line(
                        raw_x,
                        raw_y,
                        *rotate
                            + match reverse {
                                true => 90.0,
                                false => -90.0,
                            },
                        delta_y * math::UNIT_HEIGHT * speed,
                    );
                    if !in_bound(x, y) {
                        continue;
                    }
                    out.push(RendNote {
                        rend_type: 2,
                        note_type: (*note_type).into(),
                        x: x as f32,
                        y: y as f32,
                        rotate: *rotate as f32,
                        height: 0.0,
                        high_light: should_high_light,
                    });
                }
                NoteType::Hold => {
                    let head_position = floor_position - line_y;
                    let body_height =
                        hold_time * speed * seconds_per_tick - 0.0f64.max(-head_position);
                    let body_position =
                        floor_position + body_height / 2.0 - line_y + 0.0f64.max(-head_position);
                    if *time + *hold_time as i32 <= state.tick_time as i32 {
                        continue;
                    }
                    if body_position <= -body_height / 2.0 {
                        continue;
                    }
                    let Point {
                        x: temp_x,
                        y: temp_y,
                    } = math::get_pos_out_of_line(*x, *y, *rotate, position_x * math::UNIT_WIDTH);
                    let Point { x: hx, y: hy } = math::get_pos_out_of_line(
                        temp_x,
                        temp_y,
                        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
                        head_position * math::UNIT_HEIGHT,
                    );
                    let Point { x: bx, y: by } = math::get_pos_out_of_line(
                        temp_x,
                        temp_y,
                        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
                        body_position * math::UNIT_HEIGHT
                            + if body_position <= 0.0 {
                                body_height / 2.0
                            } else {
                                0.0
                            },
                    );
                    let Point { x: ex, y: ey } = math::get_pos_out_of_line(
                        temp_x,
                        temp_y,
                        math::fix_degree(rotate + if reverse { 90.0 } else { -90.0 }),
                        (body_position + body_height / 2.0) * math::UNIT_HEIGHT,
                    );
                    out_hold.push(RendNote {
                        rend_type: 2,
                        note_type: 7,
                        x: ex as f32,
                        y: ey as f32,
                        rotate: *rotate as f32,
                        height: 0.0,
                        high_light: 0,
                    });
                    out_hold.push(RendNote {
                        rend_type: 2,
                        note_type: 6,
                        x: bx as f32,
                        y: by as f32,
                        rotate: (*rotate + if reverse { 180.0 } else { 0.0 }) as f32,
                        height: (body_height * math::UNIT_HEIGHT) as f32,
                        high_light: should_high_light,
                    });
                    if *time > state.tick_time as i32 {
                        out_hold.push(RendNote {
                            rend_type: 2,
                            note_type: 5,
                            x: hx as f32,
                            y: hy as f32,
                            rotate: *rotate as f32,
                            height: 0.0,
                            high_light: should_high_light,
                        });
                    }
                }
            };
        }
    };
    process(&state.notes_above_state, false, &mut vec, &mut hold_vec);
    process(&state.notes_below_state, true, &mut vec, &mut hold_vec);
    (vec, hold_vec)
}
