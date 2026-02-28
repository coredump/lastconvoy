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

        Ok(Self {
            texture,
            anim,
            tag_pingpong,
            tag_frame_count,
            tag_frame_dur,
            timer: 0.0,
            current_frame: 0,
            pp_dir: 1,
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

    /// Switch to animation by index. Resets frame and pingpong state.
    pub fn set_animation(&mut self, index: usize) {
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
#[allow(non_snake_case)]
struct AseMeta {
    frameTags: Vec<AseFrameTag>,
}

#[derive(Deserialize)]
struct AseFrameTag {
    name: String,
    from: u32,
    to: u32,
    direction: String,
}
