#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbType {
    Defense,
    Drone,
    ShotType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbPhase {
    /// Has HP; must be shot to activate.
    Inactive,
    /// HP depleted; can now be type-cycled.
    Active,
}

pub struct Orb {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub orb_type: OrbType,
    pub phase: OrbPhase,
}

impl Orb {
    pub fn new(x: f32, y: f32, hp: i32) -> Self {
        Self {
            x,
            y,
            hp,
            orb_type: OrbType::Defense,
            phase: OrbPhase::Inactive,
        }
    }
}
