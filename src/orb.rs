// Orb entity: types, two-phase activation, and collection.
// crate::config
use crate::config::{ORB_ACTIVATION_DECAY_PER_SEC, ORB_ACTIVATION_HIT_COUNT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbType {
    Burst,
    Damage,
    Shield,
    Drone,
    DroneRemote,
    Explosive,
    FireRate,
    Pierce,
    Stagger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbPhase {
    /// Progress fills as player shoots; decays when not hit.
    Inactive,
    /// Activation complete; can be collected by ship touch.
    Active,
}

pub struct Orb {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub speed: f32,
    pub orb_type: OrbType,
    pub phase: OrbPhase,
    pub collected: bool,
    /// 0.0–1.0; reaches 1.0 to activate. Decays when not hit this frame.
    pub activation_progress: f32,
    /// Set by collision code each frame; consumed in update().
    pub hit_this_frame: bool,
    /// Accumulates time when inactive and not being hit (for seal blink effect).
    pub decay_blink_timer: f32,
}

impl Orb {
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed: f32, orb_type: OrbType) -> Self {
        Self {
            x,
            y,
            width,
            height,
            speed,
            orb_type,
            phase: OrbPhase::Inactive,
            collected: false,
            activation_progress: 0.0,
            hit_this_frame: false,
            decay_blink_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.x -= self.speed * dt;

        if self.phase == OrbPhase::Inactive {
            if self.hit_this_frame {
                self.activation_progress += 1.0 / ORB_ACTIVATION_HIT_COUNT;
                self.decay_blink_timer = 0.0;
            } else {
                self.activation_progress -= dt * ORB_ACTIVATION_DECAY_PER_SEC;
                if self.activation_progress > 0.0 {
                    self.decay_blink_timer += dt;
                } else {
                    self.decay_blink_timer = 0.0;
                }
            }
            self.activation_progress = self.activation_progress.clamp(0.0, 1.0);
            if self.activation_progress >= 1.0 {
                self.phase = OrbPhase::Active;
            }
        }

        self.hit_this_frame = false;
    }

    pub fn is_off_screen(&self) -> bool {
        self.x + self.width < 0.0
    }

    pub fn is_collected(&self) -> bool {
        self.collected
    }
}
