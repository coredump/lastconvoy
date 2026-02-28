pub struct Drone {
    pub x: f32,
    pub y: f32,
    pub fire_timer: f32,
    /// True = moves with the player; false = temporary cross-lane drone.
    pub attached: bool,
    /// Time-to-live in seconds (used for detached drones; ignored when attached).
    pub ttl: f32,
}

impl Drone {
    pub fn new(x: f32, y: f32, attached: bool, ttl: f32) -> Self {
        Self {
            x,
            y,
            fire_timer: 0.0,
            attached,
            ttl,
        }
    }
}
