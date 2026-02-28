use macroquad::prelude::*;

use crate::config::{PROJECTILE_H, PROJECTILE_W, SCREEN_W};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileSource {
    Player,
    Drone,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub source: ProjectileSource,
    pub alive: bool,
}

impl Projectile {
    pub fn new(x: f32, y: f32, speed: f32, source: ProjectileSource) -> Self {
        Self {
            x,
            y,
            speed,
            source,
            alive: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.speed * dt;
    }

    pub fn is_off_screen(&self) -> bool {
        self.x > SCREEN_W as f32
    }

    pub fn should_remove(&self) -> bool {
        !self.alive || self.is_off_screen()
    }

    pub fn draw(&self) {
        draw_rectangle(self.x, self.y, PROJECTILE_W, PROJECTILE_H, YELLOW);
    }
}
