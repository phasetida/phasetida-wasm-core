#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

enum Gx {
    I,
    II,
    III,
    IV,
}

pub static WORLD_WIDTH: f64 = 1920.0;
pub static WORLD_HEIGHT: f64 = 1080.0;
pub static UNIT_WIDTH: f64 = WORLD_WIDTH / 18.0;
pub static UNIT_HEIGHT: f64 = WORLD_HEIGHT * 0.6;

fn get_gx(valid_degree: f64) -> Gx {
    match valid_degree {
        315.0..=360.0 | 0.0..=45.0 => Gx::I,
        45.0..=135.0 => Gx::II,
        135.0..=225.0 => Gx::III,
        225.0..=315.0 => Gx::IV,
        _ => panic!(""),
    }
}

pub fn get_cross_point_with_screen(line_x: f64, line_y: f64, valid_degree: f64) -> Point {
    let gx = get_gx(valid_degree);
    let rad = valid_degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    let tan_cot = match gx {
        Gx::I | Gx::III => sin / cos,
        Gx::II | Gx::IV => cos / sin,
    };
    match gx {
        Gx::I => Point {
            x: WORLD_WIDTH,
            y: line_y + (WORLD_WIDTH - line_x) * tan_cot,
        },
        Gx::II => Point {
            x: line_x + tan_cot * (WORLD_HEIGHT - line_y),
            y: WORLD_HEIGHT,
        },
        Gx::III => Point {
            x: 0.0,
            y: line_y - line_x * tan_cot,
        },
        Gx::IV => Point {
            x: line_x - line_y * tan_cot,
            y: 0.0,
        },
    }
}

pub fn get_pos_out_of_line(line_x: f64, line_y: f64, any_degree: f64, distance: f64) -> Point {
    let rad = any_degree.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let da = cos * distance;
    let db = sin * distance;
    Point {
        x: line_x + da,
        y: line_y + db,
    }
}

pub fn fix_degree(any_degree: f64) -> f64 {
    match any_degree {
        f if f < 0.0 => fix_degree(f + 360.0),
        f if f > 360.0 => fix_degree(f - 360.0),
        f => f,
    }
}

pub fn is_point_in_judge_range(
    line_x: f64,
    line_y: f64,
    valid_degree: f64,
    point_x: f64,
    point_y: f64,
    judge_width: f64,
) -> bool {
    let gx = get_gx(valid_degree);
    let rad = valid_degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    let (p1, p2) = match gx {
        Gx::I | Gx::III => {
            let cot_or_tan = sin / cos;
            let ld1 = judge_width / cos;
            let d = point_y - line_y;
            let ld2 = d * cot_or_tan;
            let p1 = line_x - (ld2 + ld1);
            let p2 = line_x - (ld2 - ld1);
            (p1, p2)
        }
        Gx::II | Gx::IV => {
            let cot_or_tan = cos / sin;
            let ld1 = judge_width / sin;
            let d = point_y - line_y;
            let ld2 = d * cot_or_tan;
            let p1 = line_y + (ld2 + ld1);
            let p2 = line_y + (ld2 - ld1);
            (p1, p2)
        }
    };
    match gx {
        Gx::I => point_x >= p1 && point_x <= p2,
        Gx::III => point_x >= p2 && point_x <= p1,
        Gx::II => point_y >= p2 && point_y <= p1,
        Gx::IV => point_y >= p1 && point_y <= p2,
    }
}

pub fn get_pos_point_vertical_in_line(
    line_x: f64,
    line_y: f64,
    degree: f64,
    point_x: f64,
    point_y: f64,
) -> Point {
    let gx = get_gx(degree);
    let rad = degree.to_radians();
    let sin = rad.sin();
    let cos = rad.cos();
    match gx {
        Gx::I | Gx::III => {
            let tan = sin / cos;
            let tmp = point_y - line_y - (point_x - line_x) * tan;
            Point {
                x: point_x + tmp * cos * sin,
                y: point_y - tmp * cos * cos,
            }
        }
        Gx::II | Gx::IV => {
            let cot = cos / sin;
            let tmp = point_x - line_x - (point_y - line_y) * cot;
            Point {
                x: point_x - tmp * sin * sin,
                y: point_y + tmp * sin * cos,
            }
        }
    }
}
