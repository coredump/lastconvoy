use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimMode {
    Forward,
    PingPong,
}

pub struct SpriteSheet {
    pub texture: Texture2D,
    pub frame_w: f32,
    pub frame_h: f32,
    pub frame_count: usize,
    pub frame_duration: f32,
    pub anim_mode: AnimMode,
    timer: f32,
    current_frame: usize,
    pingpong_dir: i8, // +1 forward, -1 reverse
}

impl SpriteSheet {
    pub fn new(
        texture: Texture2D,
        frame_w: f32,
        frame_h: f32,
        frame_count: usize,
        frame_duration: f32,
        anim_mode: AnimMode,
    ) -> Self {
        texture.set_filter(FilterMode::Nearest);
        Self {
            texture,
            frame_w,
            frame_h,
            frame_count,
            frame_duration,
            anim_mode,
            timer: 0.0,
            current_frame: 0,
            pingpong_dir: 1,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.frame_duration {
            self.timer -= self.frame_duration;
            match self.anim_mode {
                AnimMode::Forward => {
                    self.current_frame = (self.current_frame + 1) % self.frame_count;
                }
                AnimMode::PingPong => {
                    let next = self.current_frame as i32 + self.pingpong_dir as i32;
                    if next < 0 {
                        self.pingpong_dir = 1;
                        self.current_frame = 1.min(self.frame_count - 1);
                    } else if next >= self.frame_count as i32 {
                        self.pingpong_dir = -1;
                        self.current_frame = self.frame_count.saturating_sub(2);
                    } else {
                        self.current_frame = next as usize;
                    }
                }
            }
        }
    }

    pub fn draw(&self, x: f32, y: f32) {
        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    0.0,
                    self.current_frame as f32 * self.frame_h,
                    self.frame_w,
                    self.frame_h,
                )),
                dest_size: Some(Vec2::new(self.frame_w, self.frame_h)),
                ..Default::default()
            },
        );
    }
}
