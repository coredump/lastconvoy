// Keyboard and touch input aggregation into a vertical axis.
// macroquad, crate::config
use macroquad::input::{KeyCode, TouchPhase, is_key_down, touches};
use macroquad::time::get_time;
use macroquad::window::{screen_height, screen_width};

use crate::config::{Config, PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_X, TOUCH_STRIP_WIDTH_FRAC};

const TOUCH_FULL_DEFLECTION_PX: f32 = 24.0;
const TAP_MAX_DURATION_SECS: f64 = 0.3;
const TAP_MAX_DRAG_PX: f32 = 10.0;

struct TouchTracking {
    id: u64,
    start_x: f32,
    start_y: f32,
    start_time: f64,
    is_movement: bool,
    is_drag: bool,
}

pub struct InputState {
    pub axis: f32,
    pub touch_tapped: bool,
    pub touch_tapped_pos: Option<(f32, f32)>,
    pub touch_target_y: Option<f32>,
    touch_tracking: Option<TouchTracking>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            axis: 0.0,
            touch_tapped: false,
            touch_tapped_pos: None,
            touch_target_y: None,
            touch_tracking: None,
        }
    }

    pub fn update(
        &mut self,
        config: &Config,
        portrait: bool,
        player_y: f32,
        scale: u32,
        offset_x: f32,
        offset_y: f32,
    ) {
        self.touch_tapped = false;
        self.touch_tapped_pos = None;
        self.touch_target_y = None;

        let keyboard_axis = self.read_keyboard(config);
        let touch_axis = self.read_touch(portrait, player_y, scale, offset_x, offset_y);

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

    fn screen_to_game(sx: f32, sy: f32, scale: u32, offset_x: f32, offset_y: f32) -> (f32, f32) {
        let s = scale as f32;
        ((sx - offset_x) / s, (sy - offset_y) / s)
    }

    fn read_touch(
        &mut self,
        portrait: bool,
        player_y: f32,
        scale: u32,
        offset_x: f32,
        offset_y: f32,
    ) -> f32 {
        let strip_threshold = if portrait {
            screen_height() * (1.0 - TOUCH_STRIP_WIDTH_FRAC)
        } else {
            screen_width() * TOUCH_STRIP_WIDTH_FRAC
        };
        let mut axis = 0.0;

        for touch in touches() {
            let pos = touch.position;
            match touch.phase {
                TouchPhase::Started => {
                    if self.touch_tracking.is_none() {
                        let in_strip = if portrait {
                            pos.y >= strip_threshold
                        } else {
                            pos.x <= strip_threshold
                        };
                        let (gx, gy) =
                            Self::screen_to_game(pos.x, pos.y, scale, offset_x, offset_y);
                        let on_player = (PLAYER_X..=PLAYER_X + PLAYER_WIDTH).contains(&gx)
                            && (player_y..=player_y + PLAYER_HEIGHT).contains(&gy);
                        let is_drag = on_player;
                        self.touch_tracking = Some(TouchTracking {
                            id: touch.id,
                            start_x: pos.x,
                            start_y: pos.y,
                            start_time: get_time(),
                            is_movement: in_strip || is_drag,
                            is_drag,
                        });
                    }
                }
                TouchPhase::Moved | TouchPhase::Stationary => {
                    if let Some(ref tracking) = self.touch_tracking
                        && tracking.id == touch.id
                        && tracking.is_movement
                    {
                        if tracking.is_drag {
                            let (_, gy) =
                                Self::screen_to_game(pos.x, pos.y, scale, offset_x, offset_y);
                            self.touch_target_y = Some(gy);
                        } else {
                            let delta = pos.y - tracking.start_y;
                            axis = (delta / TOUCH_FULL_DEFLECTION_PX).clamp(-1.0, 1.0);
                        }
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(ref tracking) = self.touch_tracking
                        && tracking.id == touch.id
                    {
                        let duration = get_time() - tracking.start_time;
                        let drag = ((pos.x - tracking.start_x).powi(2)
                            + (pos.y - tracking.start_y).powi(2))
                        .sqrt();
                        if duration < TAP_MAX_DURATION_SECS && drag < TAP_MAX_DRAG_PX {
                            self.touch_tapped = true;
                            self.touch_tapped_pos = Some((pos.x, pos.y));
                        }
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
