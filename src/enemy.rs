use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::{SHAKE_DURATION, SHAKE_INTENSITY};
use crate::sprite::ShakeEffect;

static NEXT_ENEMY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Small,
    Medium,
    Heavy,
    Large,
    Elite,
}

pub struct Enemy {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub max_hp: i32,
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
    pub shots_taken: i32,
    pub damage_taken: i32,
    pub knockback_timer: f32,
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
            id: NEXT_ENEMY_ID.fetch_add(1, Ordering::Relaxed),
            x,
            y,
            hp,
            max_hp: hp,
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
            shots_taken: 0,
            damage_taken: 0,
            knockback_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, knockback_speed: f32) {
        if self.knockback_timer > 0.0 {
            self.x += knockback_speed * dt;
            self.knockback_timer -= dt;
            if self.knockback_timer < 0.0 {
                self.knockback_timer = 0.0;
            }
        } else if !self.at_boundary {
            self.x -= self.speed * dt;
        }
        self.shake.update(dt);
    }

    pub fn apply_knockback(&mut self, duration: f32) {
        self.knockback_timer = duration;
        self.at_boundary = false;
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.shots_taken += 1;
        self.damage_taken += amount;
        if self.shielded && self.shield_hp > 0 {
            self.shield_hp -= amount;
            if self.shield_hp <= 0 {
                self.shielded = false;
            }
        } else {
            self.hp -= amount;
        }
        if self.kind != EnemyKind::Small {
            self.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn is_off_screen(&self) -> bool {
        self.x + self.width < 0.0
    }
}
