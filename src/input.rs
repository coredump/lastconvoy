use macroquad::input::{KeyCode, TouchPhase, is_key_down, touches};
use macroquad::window::screen_width;

use crate::config::{Config, TOUCH_STRIP_WIDTH_FRAC};

/// Pixels of vertical drag that map to full axis deflection (±1.0).
const TOUCH_FULL_DEFLECTION_PX: f32 = 60.0;

struct TouchTracking {
    id: u64,
    start_y: f32,
}

/// Normalised vertical axis: -1 = up, 0 = neutral, +1 = down.
pub struct InputState {
    pub axis: f32,
    touch_tracking: Option<TouchTracking>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            axis: 0.0,
            touch_tracking: None,
        }
    }

    /// Update axis from all input sources. Call once per frame before any gameplay logic.
    pub fn update(&mut self, config: &Config) {
        let keyboard_axis = self.read_keyboard(config);

        // TODO(Phase 2): add gamepad support via the `gilrs` crate or equivalent.
        // macroquad 0.4 removed its built-in gamepad API.

        let touch_axis = self.read_touch();

        // Keyboard takes priority; fall back to touch.
        self.axis = if keyboard_axis != 0.0 {
            keyboard_axis
        } else {
            touch_axis
        };
    }

    fn read_keyboard(&self, config: &Config) -> f32 {
        let (up1, up2, down1, down2) = if config.rotate_input {
            (KeyCode::A, KeyCode::Left, KeyCode::D, KeyCode::Right)
        } else {
            (KeyCode::W, KeyCode::Up, KeyCode::S, KeyCode::Down)
        };

        let up = is_key_down(up1) || is_key_down(up2);
        let down = is_key_down(down1) || is_key_down(down2);

        match (up, down) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        }
    }

    fn read_touch(&mut self) -> f32 {
        let strip_x_limit = screen_width() * TOUCH_STRIP_WIDTH_FRAC;
        let mut axis = 0.0;

        for touch in touches() {
            match touch.phase {
                TouchPhase::Started => {
                    if touch.position.x <= strip_x_limit && self.touch_tracking.is_none() {
                        self.touch_tracking = Some(TouchTracking {
                            id: touch.id,
                            start_y: touch.position.y,
                        });
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if let Some(ref tracking) = self.touch_tracking
                        && tracking.id == touch.id
                    {
                        let delta = touch.position.y - tracking.start_y;
                        axis = (delta / TOUCH_FULL_DEFLECTION_PX).clamp(-1.0, 1.0);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(ref tracking) = self.touch_tracking
                        && tracking.id == touch.id
                    {
                        self.touch_tracking = None;
                    }
                }
            }
        }

        axis
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
