use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Chart {
    #[serde(rename = "formatVersion")]
    pub _format_version: i32,
    #[serde(rename = "offset")]
    pub _offset: f64,
    #[serde(rename = "judgeLineList")]
    pub judge_line_list: Vec<JudgeLine>,
}

#[derive(Deserialize, Clone)]
pub struct JudgeLine {
    pub bpm: f64,
    #[serde(rename = "notesAbove")]
    pub notes_above: Vec<Note>,
    #[serde(rename = "notesBelow")]
    pub notes_below: Vec<Note>,
    #[serde(rename = "speedEvents")]
    pub speed_events: Vec<Event1>,
    #[serde(rename = "judgeLineMoveEvents")]
    pub move_events: Vec<Event4>,
    #[serde(rename = "judgeLineRotateEvents")]
    pub rotate_events: Vec<Event2>,
    #[serde(rename = "judgeLineDisappearEvents")]
    pub alpha_events: Vec<Event2>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NoteType {
    Tap,
    Drag,
    Hold,
    Flick,
}

impl TryFrom<i32> for NoteType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NoteType::Tap),
            2 => Ok(NoteType::Drag),
            3 => Ok(NoteType::Hold),
            4 => Ok(NoteType::Flick),
            _ => Err(()),
        }
    }
}

impl From<NoteType> for i8 {
    fn from(value: NoteType) -> Self {
        match value {
            NoteType::Tap => 1,
            NoteType::Drag => 2,
            NoteType::Hold => 3,
            NoteType::Flick => 4,
        }
    }
}

impl<'de> Deserialize<'de> for NoteType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = i32::deserialize(deserializer)?;
        Self::try_from(i).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Signed(0), &"1, 2, 3 or 4")
        })
    }
}

#[derive(Deserialize, Clone)]
pub struct Note {
    #[serde(rename = "type")]
    pub note_type: NoteType,
    pub time: i32,
    #[serde(rename = "positionX")]
    pub position_x: f64,
    #[serde(rename = "holdTime")]
    pub hold_time: f64,
    pub speed: f64,
    #[serde(rename = "floorPosition")]
    pub floor_position: f64,
}

#[derive(Deserialize, Clone)]
pub struct Event1 {
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub value: f64,
}

#[derive(Deserialize, Clone)]
pub struct Event2 {
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub start: f64,
    pub end: f64,
}

#[derive(Deserialize, Clone)]
pub struct Event4 {
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub start: f64,
    pub end: f64,
    pub start2: f64,
    pub end2: f64,
}

pub trait WithValue<T> {
    fn get_value(&self) -> (T, T);
    fn zero() -> (T, T);
}

impl WithValue<f64> for Event1 {
    fn get_value(&self) -> (f64, f64) {
        (self.value, self.value)
    }
    fn zero() -> (f64, f64) {
        (0.0, 0.0)
    }
}

impl WithValue<f64> for Event2 {
    fn get_value(&self) -> (f64, f64) {
        (self.start, self.end)
    }
    fn zero() -> (f64, f64) {
        (0.0, 0.0)
    }
}

impl WithValue<(f64, f64)> for Event4 {
    fn get_value(&self) -> ((f64, f64), (f64, f64)) {
        ((self.start, self.end), (self.start2, self.end2))
    }
    fn zero() -> ((f64, f64), (f64, f64)) {
        ((0.0, 0.0), (0.0, 0.0))
    }
}

#[derive(PartialEq)]
pub enum TimeState {
    Early,
    During(f64),
    Late,
}

pub trait WithTimeRange {
    fn time_start(&self) -> f64;
    fn time_end(&self) -> f64;
    fn time_length(&self) -> f64 {
        self.time_end() - self.time_start()
    }
    fn check_time(&self, time: f64) -> TimeState {
        match time {
            x if x < self.time_start() => TimeState::Early,
            x if x > self.time_end() => TimeState::Late,
            x => TimeState::During((x - self.time_start()) / self.time_length()),
        }
    }
}

impl WithTimeRange for Event1 {
    fn time_start(&self) -> f64 {
        self.start_time
    }
    fn time_end(&self) -> f64 {
        self.end_time
    }
}

impl WithTimeRange for Event2 {
    fn time_start(&self) -> f64 {
        self.start_time
    }
    fn time_end(&self) -> f64 {
        self.end_time
    }
}

impl WithTimeRange for Event4 {
    fn time_start(&self) -> f64 {
        self.start_time
    }
    fn time_end(&self) -> f64 {
        self.end_time
    }
}
