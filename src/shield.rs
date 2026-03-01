pub const SHIELD_FLASH_DURATION: f32 = 0.3;
pub const MAX_SHIELD_SEGMENTS: usize = 3;

use crate::config::{SHAKE_DURATION, SHAKE_INTENSITY};
use crate::sprite::ShakeEffect;

pub struct ShieldSegment {
    pub active: bool,
    /// Counts down after loss for a brief visual flash (seconds).
    pub flash_timer: f32,
    /// When true, this segment breaks last and triggers an explosion on break.
    pub explosive: bool,
}

impl ShieldSegment {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            flash_timer: 0.0,
            explosive: false,
        }
    }

    pub fn new_explosive() -> Self {
        Self {
            active: true,
            flash_timer: 0.0,
            explosive: true,
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

/// Result of absorbing one hit.
pub enum ShieldHitResult {
    /// No segments — player should die.
    NoShield,
    /// A normal segment was consumed.
    NormalAbsorbed,
    /// The explosive segment was consumed — trigger explosion.
    ExplosiveBreak,
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

    /// Add `count` new normal active segments, capped at `MAX_SHIELD_SEGMENTS`.
    pub fn add_segments(&mut self, count: u32) {
        let space = MAX_SHIELD_SEGMENTS.saturating_sub(self.segments.len());
        let to_add = (count as usize).min(space);
        for _ in 0..to_add {
            self.segments.push(ShieldSegment::new(true));
        }
    }

    /// Convert the last non-explosive segment to explosive.
    /// If an explosive segment already exists, the extra explosive orb is treated as a normal shield.
    /// INVARIANT: called at most once per frame (see game.rs orb-collection comment).
    pub fn convert_to_explosive(&mut self) {
        // Only one explosive segment is meaningful. Extra explosive orbs become normal shields.
        if self.has_explosive() {
            self.add_segments(1);
            return;
        }
        // Find a non-explosive segment (search from end) and convert it.
        if let Some(seg) = self.segments.iter_mut().rev().find(|s| !s.explosive) {
            seg.explosive = true;
        } else if self.segments.len() < MAX_SHIELD_SEGMENTS {
            // No segments at all; grant a fresh explosive one.
            self.segments.push(ShieldSegment::new_explosive());
        }
    }

    /// Returns true if any segment is explosive.
    pub fn has_explosive(&self) -> bool {
        self.segments.iter().any(|s| s.explosive)
    }

    /// Absorb one hit. Normal segments break first; the explosive segment breaks last.
    /// Returns a `ShieldHitResult` indicating what happened.
    pub fn take_hit(&mut self) -> ShieldHitResult {
        if self.segments.is_empty() {
            return ShieldHitResult::NoShield;
        }
        // Find the last non-explosive segment to remove first.
        let idx = self
            .segments
            .iter()
            .rposition(|s| !s.explosive)
            .unwrap_or(self.segments.len() - 1); // fall back to the explosive segment

        let seg = self.segments.remove(idx);
        self.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);

        if seg.explosive {
            ShieldHitResult::ExplosiveBreak
        } else {
            ShieldHitResult::NormalAbsorbed
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
