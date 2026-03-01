use macroquad::prelude::*;

use crate::config::{BURST_PROJECTILE_H, BURST_PROJECTILE_W, PROJECTILE_H, PROJECTILE_W, SCREEN_W};

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
    pub is_burst: bool,
    pub pierce_remaining: i32,
    pub hit_enemies: Vec<u64>,
}

impl Projectile {
    pub fn new(
        x: f32,
        y: f32,
        speed: f32,
        source: ProjectileSource,
        is_burst: bool,
        pierce_remaining: i32,
    ) -> Self {
        Self {
            x,
            y,
            speed,
            source,
            alive: true,
            is_burst,
            pierce_remaining,
            hit_enemies: Vec::new(),
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
        let (color, w, h) = if self.is_burst {
            (Color::new(1.0, 0.6, 0.2, 1.0), BURST_PROJECTILE_W, BURST_PROJECTILE_H)
        } else {
            (Color::new(0.4, 0.9, 1.0, 1.0), PROJECTILE_W, PROJECTILE_H)
        };
        draw_rectangle(self.x, self.y, w, h, color);
    }
}
