use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Per-entity horizontal shake effect for hit feedback.
pub struct ShakeEffect {
    timer: f32,
    intensity: f32,
    duration: f32,
}

impl ShakeEffect {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            intensity: 0.0,
            duration: 1.0,
        }
    }

    /// Start (or restart) a shake with the given pixel intensity and duration.
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.timer = duration;
    }

    /// Advance the shake timer by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
    }

    /// Returns true while the shake is still playing.
    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    /// Returns a horizontal pixel offset for the current frame. 0.0 when inactive.
    pub fn offset_x(&self) -> f32 {
        if self.timer <= 0.0 {
            return 0.0;
        }
        let frac = self.timer / self.duration;
        let sign = if (self.timer * 1000.0) as i32 % 2 == 0 {
            1.0_f32
        } else {
            -1.0_f32
        };
        sign * self.intensity * frac
    }
}

/// Per-entity color flash effect for hit and windup feedback.
pub struct FlashEffect {
    timer: f32,
    duration: f32,
    cooldown_timer: f32,
    color: Color,
}

impl FlashEffect {
    pub fn new() -> Self {
        Self {
            timer: 0.0,
            duration: 1.0,
            cooldown_timer: 0.0,
            color: WHITE,
        }
    }

    /// Start a flash if the cooldown has expired.
    pub fn trigger(&mut self, color: Color, duration: f32, cooldown: f32) {
        if self.cooldown_timer > 0.0 {
            return;
        }
        self.color = color;
        self.duration = duration;
        self.timer = duration;
        self.cooldown_timer = cooldown;
    }

    /// Advance timers by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.timer = (self.timer - dt).max(0.0);
        self.cooldown_timer = (self.cooldown_timer - dt).max(0.0);
    }

    /// Returns the tint color for this frame. WHITE when inactive.
    pub fn tint(&self) -> Color {
        if self.timer <= 0.0 { WHITE } else { self.color }
    }
}

/// A sprite with one or more named animations, loaded from an Aseprite JSON export.
///
/// Layout contract: each animation tag occupies its own row in the sheet, with frames
/// advancing left-to-right. Matches what `AnimatedSprite` expects and what Aseprite
/// exports when using "Export Sprite Sheet" → Array format with tags.
///
/// Supports forward and pingpong animation modes. Timing is driven by `update(dt)`.
pub struct Sprite {
    pub texture: Texture2D,
    anim: AnimatedSprite,
    /// Which tags use pingpong direction.
    tag_pingpong: Vec<bool>,
    /// Frame count per tag.
    tag_frame_count: Vec<u32>,
    /// Frame duration in seconds per tag.
    tag_frame_dur: Vec<f32>,
    // Animation state — managed manually to support pingpong + external dt.
    timer: f32,
    current_frame: u32,
    pp_dir: i8, // +1 forward, -1 reverse (pingpong only)
    /// Named slices from Aseprite export, in frame-local coordinates.
    pub slices: HashMap<String, Rect>,
    /// Tile dimensions (pixels per frame).
    pub tile_w: u32,
    pub tile_h: u32,
}

impl Sprite {
    pub async fn from_json(json_path: &str) -> Result<Self, String> {
        let text = load_string(json_path)
            .await
            .map_err(|e| format!("Failed to load {json_path}: {e}"))?;
        let parsed: AsepriteJson =
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse {json_path}: {e}"))?;

        let first = parsed
            .frames
            .values()
            .next()
            .ok_or_else(|| format!("No frames in {json_path}"))?;
        let tile_w = first.frame.w as u32;
        let tile_h = first.frame.h as u32;
        // All frames in the sheet share the same duration for now.
        let frame_dur = first.duration as f32 / 1000.0;
        let fps = ((1.0 / frame_dur).round() as u32).max(1);

        let total_frames = parsed.frames.len() as u32;

        let (anims, tag_pingpong, tag_frame_count, tag_frame_dur) =
            if parsed.meta.frameTags.is_empty() {
                (
                    vec![Animation {
                        name: "default".to_string(),
                        row: 0,
                        frames: total_frames,
                        fps,
                    }],
                    vec![false],
                    vec![total_frames],
                    vec![frame_dur],
                )
            } else {
                let mut anims = Vec::new();
                let mut pingpong = Vec::new();
                let mut counts = Vec::new();
                let mut durs = Vec::new();
                for (i, tag) in parsed.meta.frameTags.iter().enumerate() {
                    let count = tag.to - tag.from + 1;
                    anims.push(Animation {
                        name: tag.name.clone(),
                        row: i as u32,
                        frames: count,
                        fps,
                    });
                    pingpong.push(tag.direction == "pingpong");
                    counts.push(count);
                    durs.push(frame_dur);
                }
                (anims, pingpong, counts, durs)
            };

        let mut anim = AnimatedSprite::new(tile_w, tile_h, &anims, false);
        anim.set_frame(0);

        let texture_path = json_path.replace(".json", ".png");
        let texture = load_texture(&texture_path)
            .await
            .map_err(|e| format!("Failed to load texture {texture_path}: {e}"))?;
        texture.set_filter(FilterMode::Nearest);

        let mut slices = HashMap::new();
        for s in &parsed.meta.slices {
            if let Some(key) = s.keys.first() {
                slices.insert(
                    s.name.clone(),
                    Rect::new(key.bounds.x, key.bounds.y, key.bounds.w, key.bounds.h),
                );
            }
        }

        Ok(Self {
            texture,
            anim,
            tag_pingpong,
            tag_frame_count,
            tag_frame_dur,
            timer: 0.0,
            current_frame: 0,
            pp_dir: 1,
            slices,
            tile_w,
            tile_h,
        })
    }

    /// Advance the animation by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        let tag_idx = self.anim.current_animation();
        let frame_dur = self.tag_frame_dur[tag_idx];
        self.timer += dt;
        while self.timer >= frame_dur {
            self.timer -= frame_dur;
            let count = self.tag_frame_count[tag_idx];
            if self.tag_pingpong[tag_idx] {
                let next = self.current_frame as i32 + self.pp_dir as i32;
                if next >= count as i32 {
                    self.pp_dir = -1;
                    self.current_frame = count.saturating_sub(2);
                } else if next < 0 {
                    self.pp_dir = 1;
                    self.current_frame = 1.min(count - 1);
                } else {
                    self.current_frame = next as u32;
                }
            } else {
                self.current_frame = (self.current_frame + 1) % count;
            }
            self.anim.set_frame(self.current_frame);
        }
    }

    /// Switch to animation by index. Resets frame and pingpong state only if the animation changed.
    pub fn set_animation(&mut self, index: usize) {
        if self.anim.current_animation() == index {
            return;
        }
        self.anim.set_animation(index);
        self.anim.set_frame(0);
        self.current_frame = 0;
        self.pp_dir = 1;
        self.timer = 0.0;
    }

    /// Draw the current frame at (x, y) with no tint.
    pub fn draw(&self, x: f32, y: f32) {
        self.draw_tinted(x, y, WHITE);
    }

    /// Draw the current frame at (x, y) flipped vertically.
    pub fn draw_flipped_y(&self, x: f32, y: f32) {
        let f = self.anim.frame();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(f.dest_size),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    /// Draw the current frame at (x, y), clipped to `clip_h` pixels tall (from the top).
    pub fn draw_clipped_h(&self, x: f32, y: f32, clip_h: f32) {
        let f = self.anim.frame();
        let mut src = f.source_rect;
        src.h = clip_h;
        draw_texture_ex(
            &self.texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(f.dest_size.x, clip_h)),
                ..Default::default()
            },
        );
    }

    /// Draw the current frame at (x, y) with a color tint.
    pub fn draw_tinted(&self, x: f32, y: f32, tint: Color) {
        let f = self.anim.frame();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(f.dest_size),
                ..Default::default()
            },
        );
    }

    /// Draw the current frame additively using `material`, then restore the default material.
    /// The `color` alpha is scaled to `intensity` (0.0–1.0) for the overlay brightness.
    pub fn draw_additive(&self, x: f32, y: f32, color: Color, intensity: f32, material: &Material) {
        let f = self.anim.frame();
        let tint = Color::new(color.r, color.g, color.b, color.a * intensity);
        gl_use_material(material);
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(f.dest_size),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    /// Draw frame 0 of the current animation scaled to `w`×`h` with a color tint.
    pub fn draw_frozen_scaled(&mut self, x: f32, y: f32, w: f32, h: f32, tint: Color) {
        let saved = self.current_frame;
        self.anim.set_frame(0);
        let f = self.anim.frame();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
        self.anim.set_frame(saved);
    }

    /// Draw frame 0 of the current animation with a color tint (animation frozen).
    pub fn draw_tinted_frozen(&mut self, x: f32, y: f32, tint: Color) {
        let saved = self.current_frame;
        self.anim.set_frame(0);
        let f = self.anim.frame();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(f.dest_size),
                ..Default::default()
            },
        );
        self.anim.set_frame(saved);
    }

    /// Draw an arbitrary frame of the current animation tag without affecting animation state.
    pub fn draw_frame(&mut self, x: f32, y: f32, frame: u32, tint: Color) {
        let saved = self.current_frame;
        self.anim.set_frame(frame);
        let f = self.anim.frame();
        draw_texture_ex(
            &self.texture,
            x,
            y,
            tint,
            DrawTextureParams {
                source: Some(f.source_rect),
                dest_size: Some(f.dest_size),
                ..Default::default()
            },
        );
        self.anim.set_frame(saved);
    }

    /// Draw the sprite using 3-slice vertical scaling.
    ///
    /// The top and bottom slices are drawn at their natural height; the middle slice
    /// repeats (with partial clipping on the last tile) to fill the gap.
    /// All slice rects are frame-local (relative to the top-left of the frame).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_3slice_vertical(
        &self,
        x: f32,
        y: f32,
        total_h: f32,
        top: &str,
        mid: &str,
        bot: &str,
        tint: Color,
    ) {
        let frame_x = self.anim.frame().source_rect.x;

        let draw_slice = |sy: f32, slice: &Rect, clip_h: f32| {
            draw_texture_ex(
                &self.texture,
                x,
                sy,
                tint,
                DrawTextureParams {
                    source: Some(Rect::new(frame_x + slice.x, slice.y, slice.w, clip_h)),
                    dest_size: Some(vec2(slice.w, clip_h)),
                    ..Default::default()
                },
            );
        };

        let top_r = self.slices.get(top).copied().unwrap_or_default();
        let mid_r = self.slices.get(mid).copied().unwrap_or_default();
        let bot_r = self.slices.get(bot).copied().unwrap_or_default();

        draw_slice(y, &top_r, top_r.h);
        draw_slice(y + total_h - bot_r.h, &bot_r, bot_r.h);

        // Fill middle
        let mid_start = y + top_r.h;
        let mid_end = y + total_h - bot_r.h;
        let mut cursor = mid_start;
        while cursor < mid_end {
            let available = mid_end - cursor;
            let tile_h = mid_r.h.min(available);
            draw_slice(cursor, &mid_r, tile_h);
            cursor += tile_h;
        }
    }

    /// Draw the sprite using 3-slice vertical scaling with HSL color-blend tint.
    ///
    /// Applies H+S from `tint`, L from the sprite texture — preserving shading
    /// while recoloring (Aseprite "Color" blend mode equivalent).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_3slice_vertical_hsl(
        &self,
        x: f32,
        y: f32,
        total_h: f32,
        top: &str,
        mid: &str,
        bot: &str,
        tint: Color,
        material: &Material,
    ) {
        gl_use_material(material);
        self.draw_3slice_vertical(x, y, total_h, top, mid, bot, tint);
        gl_use_default_material();
    }

    /// Draw a horizontal 2-slice: `front` slice drawn once at (x, y), then
    /// `tile` slice repeated rightward to fill `total_w` pixels.
    /// If `flip_y` is true the sprite is flipped vertically.
    /// Slice rects are frame-local (relative to the top-left of the frame).
    pub fn draw_front_tiled_h(
        &self,
        x: f32,
        y: f32,
        total_w: f32,
        front: &str,
        tile: &str,
        flip_y: bool,
    ) {
        let frame_x = self.anim.frame().source_rect.x;

        let draw_slice = |sx: f32, slice: &Rect, clip_w: f32| {
            draw_texture_ex(
                &self.texture,
                sx,
                y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(frame_x + slice.x, slice.y, clip_w, slice.h)),
                    dest_size: Some(vec2(clip_w, slice.h)),
                    flip_y,
                    ..Default::default()
                },
            );
        };

        let front_r = self.slices.get(front).copied().unwrap_or_default();
        let tile_r = self.slices.get(tile).copied().unwrap_or_default();

        draw_slice(x, &front_r, front_r.w);

        let tile_start = x + front_r.w;
        let tile_end = x + total_w;
        let mut cursor = tile_start;
        while cursor < tile_end {
            let available = tile_end - cursor;
            let tile_w = tile_r.w.min(available);
            draw_slice(cursor, &tile_r, tile_w);
            cursor += tile_w;
        }
    }
}

// --- Aseprite JSON deserialization ---

#[derive(Deserialize)]
struct AsepriteJson {
    frames: HashMap<String, AseFrame>,
    meta: AseMeta,
}

#[derive(Deserialize)]
struct AseFrame {
    frame: AseRect,
    duration: u32,
}

#[derive(Deserialize)]
struct AseRect {
    w: f32,
    h: f32,
}

#[derive(Deserialize)]
struct AseSliceBounds {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Deserialize)]
struct AseSliceKey {
    bounds: AseSliceBounds,
}

#[derive(Deserialize)]
struct AseSlice {
    name: String,
    keys: Vec<AseSliceKey>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct AseMeta {
    frameTags: Vec<AseFrameTag>,
    #[serde(default)]
    slices: Vec<AseSlice>,
}

#[derive(Deserialize)]
struct AseFrameTag {
    name: String,
    from: u32,
    to: u32,
    direction: String,
}
