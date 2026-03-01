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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteDroneLane {
    Top,
    Bottom,
}

pub struct RemoteDrone {
    pub x: f32,
    pub y: f32,
    pub fire_timer: f32,
    pub lane: RemoteDroneLane,
}

impl RemoteDrone {
    pub fn new(x: f32, y: f32, lane: RemoteDroneLane) -> Self {
        Self {
            x,
            y,
            fire_timer: 0.0,
            lane,
        }
    }
}
