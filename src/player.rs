use crate::config::{BOTTOM_BORDER_TOP, ENEMY_LANE_TOP, PLAYER_HEIGHT};
use crate::sprite::ShakeEffect;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
    pub fire_timer: f32,
    pub fire_rate: f32,
    pub shake: ShakeEffect,
}

impl Player {
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed: f32, fire_rate: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            speed,
            fire_timer: 0.0,
            fire_rate,
            shake: ShakeEffect::new(),
        }
    }

    pub fn update(&mut self, axis: f32, dt: f32) {
        let y_min = ENEMY_LANE_TOP as f32;
        let y_max = BOTTOM_BORDER_TOP as f32 - PLAYER_HEIGHT;
        self.y = (self.y + axis * self.speed * dt).clamp(y_min, y_max);

        self.fire_timer -= dt;
        self.shake.update(dt);
    }

    pub fn should_fire(&mut self) -> bool {
        if self.fire_timer <= 0.0 {
            self.fire_timer = self.fire_rate;
            true
        } else {
            false
        }
    }
}
