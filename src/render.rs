// Render pipeline: offscreen target, integer scaling, letterbox blit, portrait detection.
// macroquad
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::BLACK;
use macroquad::math::{Rect, vec2};
use macroquad::texture::{
    DrawTextureParams, FilterMode, RenderTarget, draw_texture_ex, render_target,
};
use macroquad::window::{clear_background, screen_height, screen_width};

use crate::config::{MAX_SCALE, MIN_SCALE, SCREEN_H, SCREEN_W};

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn get_device_portrait() -> i32;
}

pub struct RenderPipeline {
    pub target: RenderTarget,
    camera: Camera2D,
    portrait: bool,
}

impl RenderPipeline {
    pub fn new() -> Self {
        let target = render_target(SCREEN_W, SCREEN_H);
        target.texture.set_filter(FilterMode::Nearest);

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32));
        camera.render_target = Some(target.clone());

        Self {
            target,
            camera,
            portrait: false,
        }
    }

    pub fn is_portrait(&self) -> bool {
        self.portrait
    }

    pub fn begin(&self) {
        set_camera(&self.camera);
        clear_background(BLACK);
    }

    pub fn end_and_blit(&mut self) {
        self.portrait = Self::detect_portrait();

        set_default_camera();
        clear_background(BLACK);

        let scale = self.compute_scale() as f32;
        let dest_w = SCREEN_W as f32 * scale;
        let dest_h = SCREEN_H as f32 * scale;
        let offset_x = (screen_width() - dest_w) * 0.5;
        let offset_y = (screen_height() - dest_h) * 0.5;

        draw_texture_ex(
            &self.target.texture,
            offset_x,
            offset_y,
            macroquad::color::WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest_w, dest_h)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    pub fn scale(&self) -> u32 {
        self.compute_scale()
    }

    pub fn offset(&self) -> (f32, f32) {
        let scale = self.compute_scale() as f32;
        let dest_w = SCREEN_W as f32 * scale;
        let dest_h = SCREEN_H as f32 * scale;
        (
            (screen_width() - dest_w) * 0.5,
            (screen_height() - dest_h) * 0.5,
        )
    }

    fn compute_scale(&self) -> u32 {
        let scale_x = (screen_width() / SCREEN_W as f32).floor() as u32;
        let scale_y = (screen_height() / SCREEN_H as f32).floor() as u32;
        scale_x.min(scale_y).clamp(MIN_SCALE, MAX_SCALE)
    }

    fn detect_portrait() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            return unsafe { get_device_portrait() != 0 };
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            screen_height() > screen_width()
        }
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}
