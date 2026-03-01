use std::sync::atomic::{AtomicU64, Ordering};

use macroquad::prelude::{Color, WHITE};

use crate::config::{
    DAMAGE_FLASH_COLOR, DAMAGE_FLASH_COOLDOWN, DAMAGE_FLASH_DURATION, WINDUP_FLASH_COLOR,
    WINDUP_FLASH_FREQ_MAX, WINDUP_FLASH_FREQ_MIN,
};
use crate::sprite::{FlashEffect, ShakeEffect};

static NEXT_ENEMY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Small,
    Medium,
    Heavy,
    Large,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyState {
    Moving,
    Breaching,
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
    pub state: EnemyState,
    /// Total wind-up duration before breach resolves (seconds).
    pub windup_time: f32,
    /// Accumulated wind-up time while in Breaching state.
    pub windup_elapsed: f32,
    pub shielded: bool,
    pub shield_hp: i32,
    pub shake: ShakeEffect,
    /// Oscillator phase for windup flash (0..1 cycles).
    pub windup_phase: f32,
    pub shots_taken: i32,
    pub damage_taken: i32,
    /// True once this enemy has been knocked back; prevents repeated knockback.
    pub stagger_immune: bool,
    pub flash: FlashEffect,
}

impl Enemy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: f32,
        y: f32,
        kind: EnemyKind,
        hp: i32,
        speed: f32,
        width: f32,
        height: f32,
        windup_time: f32,
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
            state: EnemyState::Moving,
            windup_time,
            windup_elapsed: 0.0,
            shielded: false,
            shield_hp: 0,
            shake: ShakeEffect::new(),
            windup_phase: 0.0,
            shots_taken: 0,
            damage_taken: 0,
            stagger_immune: false,
            flash: FlashEffect::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.state == EnemyState::Moving {
            self.x -= self.speed * dt;
        }
        self.shake.update(dt);
        self.flash.update(dt);
        if self.state == EnemyState::Breaching && self.windup_time > 0.0 {
            let t = (self.windup_elapsed / self.windup_time).clamp(0.0, 1.0);
            let freq = WINDUP_FLASH_FREQ_MIN + (WINDUP_FLASH_FREQ_MAX - WINDUP_FLASH_FREQ_MIN) * t;
            self.windup_phase = (self.windup_phase + freq * dt) % 1.0;
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.shots_taken += 1;
        self.damage_taken += amount;
        self.flash.trigger(
            DAMAGE_FLASH_COLOR,
            DAMAGE_FLASH_DURATION,
            DAMAGE_FLASH_COOLDOWN,
        );
        if self.shielded && self.shield_hp > 0 {
            self.shield_hp -= amount;
            if self.shield_hp <= 0 {
                self.shielded = false;
            }
        } else {
            self.hp -= amount;
        }
    }

    /// Returns the windup flash tint based on the current oscillator phase.
    pub fn windup_tint(&self) -> Color {
        use std::f32::consts::TAU;
        if (self.windup_phase * TAU).sin() > 0.0 {
            WINDUP_FLASH_COLOR
        } else {
            WHITE
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn is_off_screen(&self) -> bool {
        self.x + self.width < 0.0
    }
}
