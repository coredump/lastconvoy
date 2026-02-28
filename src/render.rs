use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::color::BLACK;
use macroquad::math::{Rect, vec2};
use macroquad::texture::{
    DrawTextureParams, FilterMode, RenderTarget, draw_texture_ex, render_target,
};
use macroquad::window::{clear_background, screen_height, screen_width};

use crate::config::{MAX_SCALE, MIN_SCALE, SCREEN_H, SCREEN_W};

pub struct RenderPipeline {
    pub target: RenderTarget,
    camera: Camera2D,
}

impl RenderPipeline {
    pub fn new() -> Self {
        let target = render_target(SCREEN_W, SCREEN_H);
        target.texture.set_filter(FilterMode::Nearest);

        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32));
        camera.render_target = Some(target.clone());

        Self { target, camera }
    }

    /// Set the active camera to the offscreen render target and clear it.
    /// Call this before any gameplay drawing each frame.
    pub fn begin(&self) {
        set_camera(&self.camera);
        clear_background(BLACK);
    }

    /// Reset to the screen camera, compute integer scale, and blit the render
    /// target texture centred with black letterbox bars.
    /// Call this after all gameplay drawing is done.
    pub fn end_and_blit(&self) {
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
                flip_y: true, // macroquad render targets are upside-down
                ..Default::default()
            },
        );
    }

    /// Largest integer scale where both dimensions fit within the current window.
    /// Clamped to [MIN_SCALE, MAX_SCALE].
    fn compute_scale(&self) -> u32 {
        let scale_x = (screen_width() / SCREEN_W as f32).floor() as u32;
        let scale_y = (screen_height() / SCREEN_H as f32).floor() as u32;
        scale_x.min(scale_y).clamp(MIN_SCALE, MAX_SCALE)
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}
