use crate::config::{
    DRONE_HEIGHT, DRONE_Y_OFFSETS, PLAYER_HEIGHT, PLAYER_LANE_PADDING, SCREEN_H,
    TOP_UPGRADE_LANE_TOP,
};
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

    pub fn update(&mut self, axis: f32, dt: f32, has_top_drone: bool, has_bottom_drone: bool) {
        let top_drone_overhang = if has_top_drone {
            -DRONE_Y_OFFSETS[0].min(0.0) // positive: how far above player the drone extends
        } else {
            0.0
        };
        let bottom_drone_overhang = if has_bottom_drone {
            (DRONE_Y_OFFSETS[1] + DRONE_HEIGHT - PLAYER_HEIGHT).max(0.0)
        } else {
            0.0
        };
        let y_min = TOP_UPGRADE_LANE_TOP as f32 + top_drone_overhang + PLAYER_LANE_PADDING;
        let y_max = SCREEN_H as f32 - PLAYER_HEIGHT - bottom_drone_overhang - PLAYER_LANE_PADDING;
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
