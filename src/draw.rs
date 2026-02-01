use crate::chart::{Note, NoteType};
use crate::effect::{HitEffect, SplashEffect};
use crate::math::{self, Point};
use crate::renders::{
    self, Dense, RendClickEffect, RendNote, RendPoint, RendSplashEffect, RendStatistics,
};
use crate::states::{LineState, NoteScore, NoteState};
use crate::{
    CHART_STATISTICS, DRAW_IMAGE_OFFSET, HIT_EFFECT_POOL, LINE_STATES, SPLASH_EFFECT_POOL,
    TOUCH_STATES,
};

pub struct DrawImageOffset {
    pub hold_head_height: f64,
    pub hold_head_highlight_height: f64,
    pub hold_end_height: f64,
    pub hold_end_highlight_height: f64,
}

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

impl Default for DrawImageOffset {
    fn default() -> Self {
        DrawImageOffset {
            hold_head_height: 0.0,
            hold_head_highlight_height: 0.0,
            hold_end_height: 0.0,
            hold_end_highlight_height: 0.0,
        }
    }
}
pub fn load_image_offset(
    hold_head_height: f64,
    hold_head_highlight_height: f64,
    hold_end_height: f64,
    hold_end_highlight_height: f64,
) {
    DRAW_IMAGE_OFFSET.with_borrow_mut(|offset| {
        *offset = DrawImageOffset {
            hold_head_height,
            hold_head_highlight_height,
            hold_end_height,
            hold_end_highlight_height,
        };
    });
}

pub fn process_state_to_drawable(output_buffer: &js_sys::Uint8Array) {
    let mut wrapped_buffer = BufferWithCursor {
        buffer: output_buffer,
        cursor: 0,
    };
    CHART_STATISTICS.with_borrow(|statistics| {
        wrapped_buffer.write(
            RendStatistics {
                rend_type: 5,
                combo: statistics.combo,
                max_combo: statistics.max_combo,
                score: statistics.score as f32,
                accurate: statistics.accurate as f32,
            }
            .to_bytes(),
        );
    });
    LINE_STATES.with_borrow(|states| {
        DRAW_IMAGE_OFFSET.with_borrow(|offset| {
            states
                .iter()
                .for_each(|it| write_line(&mut wrapped_buffer, it));
            write_notes(&mut wrapped_buffer, states.as_ref(), offset);
        });
    });
    HIT_EFFECT_POOL.with_borrow(|effects| {
        write_click_effects(&mut wrapped_buffer, effects);
    });
    SPLASH_EFFECT_POOL.with_borrow(|effects| {
        write_splash_effects(&mut wrapped_buffer, effects);
    });
    TOUCH_STATES.with_borrow(|touches| {
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
    });
    wrapped_buffer.write(&[0]);
}

fn write_splash_effects(wrapped_buffer: &mut BufferWithCursor, states: &[SplashEffect]) {
    states.iter().for_each(|it| {
        if !it.enable {
            return;
        }
        wrapped_buffer.write(
            RendSplashEffect {
                rend_type: 6,
                x: it.x as f32,
                y: it.y as f32,
                frame: ((30.0 * it.progress).floor() as i8).clamp(0, 29),
                tint_type: it.tint_type,
            }
            .to_bytes(),
        );
    });
}

fn write_click_effects(wrapped_buffer: &mut BufferWithCursor, states: &[HitEffect]) {
    states.iter().for_each(|it| {
        if !it.enable {
            return;
        }
        wrapped_buffer.write(
            RendClickEffect {
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
    offset: &DrawImageOffset,
) {
    let notes = states
        .iter()
        .fold((Vec::new(), Vec::new()), |(v1, v2), it| {
            process_notes(it, offset, v1, v2)
        });
    notes
        .0
        .iter()
        .for_each(|it| wrapped_buffer.write(it.to_bytes()));
    notes
        .1
        .iter()
        .for_each(|it| wrapped_buffer.write(it.to_bytes()));
}

fn process_notes(
    state: &LineState,
    offset: &DrawImageOffset,
    mut vec: Vec<RendNote>,
    mut hold_vec: Vec<RendNote>,
) -> (Vec<RendNote>, Vec<RendNote>) {
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
            let should_high_light: i8 = if *highlight { 1 } else { 0 };
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
                        head_position * math::UNIT_HEIGHT
                            - (if *highlight {
                                offset.hold_head_highlight_height / 2.0
                            } else {
                                offset.hold_head_height / 2.0
                            }),
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
                        (body_position + body_height / 2.0) * math::UNIT_HEIGHT
                            + (if *highlight {
                                offset.hold_end_highlight_height / 2.0
                            } else {
                                offset.hold_end_height / 2.0
                            }),
                    );
                    out_hold.push(RendNote {
                        rend_type: 2,
                        note_type: 7,
                        x: ex as f32,
                        y: ey as f32,
                        rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 })
                            as f32,
                        height: 0.0,
                        high_light: 0,
                    });
                    out_hold.push(RendNote {
                        rend_type: 2,
                        note_type: 6,
                        x: bx as f32,
                        y: by as f32,
                        rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 })
                            as f32,
                        height: (body_height * math::UNIT_HEIGHT) as f32,
                        high_light: should_high_light,
                    });
                    if *time > state.tick_time as i32 {
                        out_hold.push(RendNote {
                            rend_type: 2,
                            note_type: 5,
                            x: hx as f32,
                            y: hy as f32,
                            rotate: math::fix_degree(*rotate + if reverse { 180.0 } else { 0.0 })
                                as f32,
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
