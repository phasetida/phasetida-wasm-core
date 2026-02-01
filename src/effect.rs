use crate::{HIT_EFFECT_POOL, SPLASH_EFFECT_POOL};

pub struct HitEffect {
    pub enable: bool,
    pub x: f64,
    pub y: f64,
    pub progress: f64,
    pub tint_type: i8,
}

pub struct SplashEffect {
    pub enable: bool,
    pub x: f64,
    pub y: f64,
    pub x_vec: f64,
    pub y_vec: f64,
    pub speed: f64,
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

impl Default for SplashEffect {
    fn default() -> Self {
        SplashEffect {
            enable: false,
            x: 0.0,
            y: 0.0,
            x_vec: 0.0,
            y_vec: 0.0,
            speed: 0.0,
            progress: 0.0,
            tint_type: 0,
        }
    }
}

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let seed = if seed == 0 { 0xdead_beef } else { seed };
        Rng { state: seed }
    }

    pub fn next(&mut self) -> f64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let x = self.state.wrapping_mul(0x2545F4914F6CDD1D);
        (x as u32) as f64 / u32::MAX as f64
    }

    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next()
    }
}

const RATE: f64 = 2.0;

pub fn tick_effect(delta_time_in_second: f64) {
    HIT_EFFECT_POOL.with_borrow_mut(|pool| {
        pool.iter_mut().for_each(|it| {
            if it.enable {
                it.progress += delta_time_in_second.max(0.0) * RATE;
                if it.progress >= 1.0 {
                    it.enable = false;
                }
            }
        });
    });
    SPLASH_EFFECT_POOL.with_borrow_mut(|pool| {
        pool.iter_mut().for_each(|it| {
            if it.enable {
                it.progress += delta_time_in_second.max(0.0) * RATE;
                if it.progress >= 1.0 {
                    it.enable = false;
                    return;
                }
                it.speed -= (it.speed * 7.0 * delta_time_in_second.max(0.0)).max(0.0);
                it.x += it.speed * it.x_vec * delta_time_in_second.max(0.0);
                it.y += it.speed * it.y_vec * delta_time_in_second.max(0.0);
            }
        });
    })
}

pub fn new_splash_effect(rng: &mut Rng, x: f64, y: f64, tint_type: i8, count: u8) {
    let mut i = count;
    SPLASH_EFFECT_POOL.with_borrow_mut(|pool| {
        for effect in pool.iter_mut() {
            if !effect.enable {
                effect.enable = true;
                effect.x = x;
                effect.y = y;
                let rand = rng.range(0.0, 2.0 * std::f64::consts::PI);
                effect.x_vec = rand.cos();
                effect.y_vec = rand.sin();
                effect.speed = 2500.0;
                effect.tint_type = tint_type;
                effect.progress = 0.0;
                i -= 1;
                if i <= 0 {
                    return;
                }
            }
        }
    });
}

pub fn new_click_effect(seed: f64, x: f64, y: f64, tint_type: i8) {
    let mut rng = Rng::new((seed * 114514.0) as u64);
    HIT_EFFECT_POOL.with_borrow_mut(|pool| {
        for effect in pool.iter_mut() {
            if !effect.enable {
                effect.enable = true;
                effect.x = x;
                effect.y = y;
                effect.progress = 0.0;
                effect.tint_type = tint_type;
                new_splash_effect(
                    &mut rng,
                    x,
                    y,
                    tint_type,
                    if effect.tint_type == 1 { 3 } else { 4 },
                );
                return;
            }
        }
    });
}
