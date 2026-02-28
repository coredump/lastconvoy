pub const SHIELD_FLASH_DURATION: f32 = 0.3;
pub const MAX_SHIELD_SEGMENTS: usize = 3;

use crate::config::{SHAKE_DURATION, SHAKE_INTENSITY};
use crate::sprite::ShakeEffect;

pub struct ShieldSegment {
    pub active: bool,
    /// Counts down after loss for a brief visual flash (seconds).
    pub flash_timer: f32,
}

impl ShieldSegment {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            flash_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.flash_timer > 0.0 {
            self.flash_timer -= dt;
        }
    }

    pub fn is_flashing(&self) -> bool {
        self.flash_timer > 0.0
    }
}

/// Manages the player's shield layers.
pub struct ShieldSystem {
    pub segments: Vec<ShieldSegment>,
    pub shake: ShakeEffect,
}

impl ShieldSystem {
    pub fn new(starting: u32) -> Self {
        let segments = (0..starting as usize)
            .map(|_| ShieldSegment::new(true))
            .collect();
        Self {
            segments,
            shake: ShakeEffect::new(),
        }
    }

    /// Add `count` new active segments, capped at `MAX_SHIELD_SEGMENTS`.
    pub fn add_segments(&mut self, count: u32) {
        let space = MAX_SHIELD_SEGMENTS.saturating_sub(self.segments.len());
        let to_add = (count as usize).min(space);
        for _ in 0..to_add {
            self.segments.push(ShieldSegment::new(true));
        }
    }

    /// Absorb one hit. Returns `true` if a segment was consumed (player survives),
    /// `false` if there were no segments (player dies).
    pub fn take_hit(&mut self) -> bool {
        if let Some(seg) = self.segments.pop() {
            let _ = seg;
            self.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
            true
        } else {
            false
        }
    }

    /// Number of active shield segments remaining.
    pub fn count(&self) -> usize {
        self.segments.len()
    }

    /// Tick flash timers on all segments and the shake effect.
    pub fn update(&mut self, dt: f32) {
        for seg in &mut self.segments {
            seg.update(dt);
        }
        self.shake.update(dt);
    }
}
