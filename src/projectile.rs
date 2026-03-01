use macroquad::prelude::*;

use crate::config::{PROJECTILE_H, PROJECTILE_W, SCREEN_W};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileSource {
    Player,
    /// Attached drone following the player — does not interact with orbs.
    Drone,
    /// Upgrade-lane remote drone — activates orbs (Phase 1 only; no cycling).
    RemoteDrone,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub source: ProjectileSource,
    pub alive: bool,
    pub pierce_remaining: i32,
    pub hit_enemies: Vec<u64>,
}

impl Projectile {
    pub fn new(
        x: f32,
        y: f32,
        speed: f32,
        source: ProjectileSource,
        pierce_remaining: i32,
    ) -> Self {
        Self {
            x,
            y,
            speed,
            source,
            alive: true,
            pierce_remaining,
            hit_enemies: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.speed * dt;
    }

    pub fn is_off_screen(&self) -> bool {
        self.x > SCREEN_W as f32 || self.x < -(PROJECTILE_W + 2.0)
    }

    pub fn should_remove(&self) -> bool {
        !self.alive || self.is_off_screen()
    }

    pub fn draw(&self) {
        draw_rectangle(
            self.x,
            self.y,
            PROJECTILE_W,
            PROJECTILE_H,
            Color::new(0.4, 0.9, 1.0, 1.0),
        );
    }
}
