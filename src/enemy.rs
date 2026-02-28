use crate::config::{SHAKE_DURATION, SHAKE_INTENSITY};
use crate::sprite::ShakeEffect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Small,
    Medium,
    Heavy,
    Large,
    Elite,
}

pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub kind: EnemyKind,
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    /// True when the enemy has stopped at the boundary (may or may not hold a slot).
    pub at_boundary: bool,
    pub damage_timer: f32,
    pub shielded: bool,
    pub shield_hp: i32,
    /// Index of the boundary slot this enemy occupies, or None if queued/not at boundary.
    pub slot_id: Option<usize>,
    pub shake: ShakeEffect,
}

impl Enemy {
    pub fn new(
        x: f32,
        y: f32,
        kind: EnemyKind,
        hp: i32,
        speed: f32,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            x,
            y,
            hp,
            kind,
            speed,
            width,
            height,
            at_boundary: false,
            damage_timer: 0.0,
            shielded: false,
            shield_hp: 0,
            slot_id: None,
            shake: ShakeEffect::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.at_boundary {
            self.x -= self.speed * dt;
        }
        self.shake.update(dt);
    }

    pub fn take_damage(&mut self, amount: i32) {
        if self.shielded && self.shield_hp > 0 {
            self.shield_hp -= amount;
            if self.shield_hp <= 0 {
                self.shielded = false;
            }
        } else {
            self.hp -= amount;
        }
        self.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn is_off_screen(&self) -> bool {
        self.x + self.width < 0.0
    }
}
