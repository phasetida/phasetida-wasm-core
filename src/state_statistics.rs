use wasm_bindgen::JsValue;

use crate::{
    CHART_STATISTICS, FLATTEN_NOTE_INDEX, LINE_STATES,
    states::{self, LineState, NoteState},
};

pub struct NoteIndex {
    pub line_index: usize,
    pub above: bool,
    pub note_index: usize,
    pub time_in_second: f64,
}

pub struct ChartStatistics {
    pub combo: u32,
    pub max_combo: u32,
    pub score: f64,
    pub accurate: f64,
}

impl Default for ChartStatistics {
    fn default() -> Self {
        ChartStatistics {
            combo: 0,
            max_combo: 0,
            score: 0.0,
            accurate: 0.0,
        }
    }
}

impl NoteIndex {
    pub fn index<'a>(&self, line_states: &'a [LineState]) -> Option<&'a NoteState> {
        line_states
            .get(self.line_index)
            .map(|it| {
                if self.above {
                    &it.notes_above_state
                } else {
                    &it.notes_below_state
                }
            })
            .and_then(|it| it.get(self.note_index))
    }
}

pub fn init_flatten_line_state() -> Result<(), JsValue> {
    LINE_STATES
        .try_with(|line_state_raw| {
            FLATTEN_NOTE_INDEX
                .try_with(|flatten_index_raw| {
                    let line_state = line_state_raw.borrow();
                    let mut flatten_index = flatten_index_raw.borrow_mut();
                    _init_flatten_line_state(line_state.as_ref(), &mut flatten_index);
                })
                .map_err(|_| "failed to access states")
        })
        .map_err(|_| "failed to access states")??;
    Ok(())
}

fn _init_flatten_line_state(line_state: &[LineState], flatten_index: &mut Vec<NoteIndex>) {
    let mut o = line_state
        .iter()
        .enumerate()
        .flat_map(|(i, it)| {
            fn flatten(
                seconds_per_tick: f64,
                notes: &[NoteState],
                above: bool,
                i: usize,
            ) -> impl std::iter::Iterator<Item = NoteIndex> {
                notes.iter().enumerate().map(move |(j, nit)| NoteIndex {
                    line_index: i,
                    above,
                    note_index: j,
                    time_in_second: (nit.note.time as f64 + nit.note.hold_time) * seconds_per_tick,
                })
            }
            let seconds_per_tick = states::get_seconds_per_tick(it.bpm);
            flatten(seconds_per_tick, &it.notes_above_state, true, i).chain(flatten(
                seconds_per_tick,
                &it.notes_below_state,
                false,
                i,
            ))
        })
        .collect::<Vec<_>>();
    o.sort_by_key(|it| (it.time_in_second * 100000.0) as i32);
    *flatten_index = o;
}

pub fn refresh_chart_statistics() -> Result<(), JsValue> {
    LINE_STATES
        .try_with(|line_states_raw| {
            FLATTEN_NOTE_INDEX
                .try_with(|flatten_index_raw| {
                    CHART_STATISTICS
                        .try_with(|chart_statistics_raw| {
                            let line_states = line_states_raw.borrow();
                            let flatten_index = flatten_index_raw.borrow();
                            let mut chart_statistics = chart_statistics_raw.borrow_mut();
                            _refresh_chart_statistics(
                                line_states.as_ref(),
                                flatten_index.as_ref(),
                                &mut chart_statistics,
                            );
                        })
                        .map_err(|_| "failed to access states")
                })
                .map_err(|_| "failed to access states")
        })
        .map_err(|_| "failed to access states")???;
    Ok(())
}

fn _refresh_chart_statistics(
    line_states: &[LineState],
    flatten_index: &[NoteIndex],
    chart_statistics: &mut ChartStatistics,
) {
    let mut combos = vec![0u32];
    flatten_index.iter().for_each(|it| {
        let state = it.index(line_states);
        match state {
            None => {}
            Some(state) => match state.score {
                states::NoteScore::Perfect | states::NoteScore::Good => {
                    combos.last_mut().map(|it| *it += 1);
                }
                states::NoteScore::Bad | states::NoteScore::Miss => {
                    combos.push(0u32);
                }
                states::NoteScore::None => {}
            },
        };
    });
    let max_combo = combos.iter().max().map(|it| *it).unwrap_or(0u32);
    let current_combo = combos.last().map(|it| *it).unwrap_or(0u32);
    let judge_results =
        flatten_index
            .iter()
            .fold((0, 0), |score, it| match it.index(line_states) {
                None => score,
                Some(state) => match state.score {
                    states::NoteScore::Perfect => (score.0 + 1, score.1),
                    states::NoteScore::Good => (score.0, score.1 + 1),
                    _ => score,
                },
            });
    let total_notes = flatten_index.len();
    let accurate = (judge_results.0 as f64 + judge_results.1 as f64 * 0.65) / total_notes as f64;
    let score = (max_combo as f64 / total_notes as f64 * 100000.0) + (accurate * 900000.0);
    *chart_statistics = ChartStatistics {
        combo: current_combo,
        max_combo,
        score,
        accurate,
    };
}
