pub struct Drone {
    pub x: f32,
    pub y: f32,
    pub fire_timer: f32,
}

impl Drone {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            fire_timer: 0.0,
        }
    }
}

pub struct RemoteDrone {
    pub x: f32,
    pub y: f32,
    pub fire_timer: f32,
}

impl RemoteDrone {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            fire_timer: 0.0,
        }
    }
}
