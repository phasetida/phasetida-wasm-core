use wasm_bindgen::JsValue;

use crate::HIT_EFFECT_POOL;

pub struct HitEffect {
    pub enable: bool,
    pub x: f64,
    pub y: f64,
    pub progress: f64,
    pub tint_type: i8,
}

impl Default for HitEffect {
    fn default() -> Self {
        HitEffect {
            enable: false,
            x: 0.0,
            y: 0.0,
            progress: 0.0,
            tint_type: 0,
        }
    }
}

const RATE: f64 = 2.0;

pub fn tick_effect(delta_time_in_second: f64) -> Result<(), JsValue> {
    HIT_EFFECT_POOL
        .try_with(|pool_ref| {
            let mut pool = pool_ref.borrow_mut();
            pool.iter_mut().for_each(|it| {
                if it.enable {
                    it.progress += delta_time_in_second.max(0.0) * RATE;
                    if it.progress >= 1.0 {
                        it.enable = false;
                    }
                }
            });
        })
        .map_err(|_| "failed to access state")?;
    Ok(())
}

pub fn new_effect(x: f64, y: f64, tint_type: i8) {
    let _ = HIT_EFFECT_POOL.try_with(|pool_ref| {
        let mut pool = pool_ref.borrow_mut();
        for effect in pool.iter_mut() {
            if !effect.enable {
                effect.enable = true;
                effect.x = x;
                effect.y = y;
                effect.progress = 0.0;
                effect.tint_type = tint_type;
                return;
            }
        }
    });
}
