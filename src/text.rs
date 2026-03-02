use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct GlyphMetrics {
    pub src: Rect,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
}

pub struct BitmapFont {
    texture: Texture2D,
    glyphs: HashMap<char, GlyphMetrics>,
    fallback: char,
    line_height: f32,
}

#[derive(Deserialize)]
struct FontDef {
    line_height: u32,
    fallback: String,
    glyphs: Vec<GlyphDef>,
}

#[derive(Deserialize)]
struct GlyphDef {
    ch: String,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    x_offset: Option<i32>,
    y_offset: Option<i32>,
    x_advance: Option<i32>,
}

impl BitmapFont {
    pub async fn load(atlas_path: &str, metrics_path: &str) -> Result<Self, String> {
        let text = load_string(metrics_path)
            .await
            .map_err(|e| format!("Failed to load font metrics {metrics_path}: {e}"))?;
        if let Ok(parsed) = serde_json::from_str::<FontDef>(&text) {
            let texture = load_texture(atlas_path)
                .await
                .map_err(|e| format!("Failed to load font atlas {atlas_path}: {e}"))?;
            texture.set_filter(FilterMode::Nearest);

            let fallback =
                parsed.fallback.chars().next().ok_or_else(|| {
                    format!("Font metrics {metrics_path} has empty fallback glyph")
                })?;

            let mut glyphs = HashMap::new();
            for g in parsed.glyphs {
                let ch = g.ch.chars().next().ok_or_else(|| {
                    format!("Font metrics {metrics_path} contains empty glyph key")
                })?;
                glyphs.insert(
                    ch,
                    GlyphMetrics {
                        src: Rect::new(g.x as f32, g.y as f32, g.w as f32, g.h as f32),
                        x_offset: g.x_offset.unwrap_or(0) as f32,
                        y_offset: g.y_offset.unwrap_or(0) as f32,
                        x_advance: g.x_advance.unwrap_or(g.w as i32) as f32,
                    },
                );
            }

            return Ok(Self {
                texture,
                glyphs,
                fallback,
                line_height: parsed.line_height as f32,
            });
        }

        // Monogram bitmap JSON format: { "A": [row_mask0, ...], ... }.
        let parsed: HashMap<String, Vec<u32>> = serde_json::from_str(&text)
            .map_err(|e| format!("Unsupported font metrics format in {metrics_path}: {e}"))?;
        Self::from_monogram(parsed, metrics_path)
    }

    fn from_monogram(
        parsed: HashMap<String, Vec<u32>>,
        metrics_path: &str,
    ) -> Result<Self, String> {
        let mut entries: Vec<(char, Vec<u32>)> = Vec::new();
        for (key, rows) in parsed {
            let mut chars = key.chars();
            let Some(ch) = chars.next() else {
                continue;
            };
            if chars.next().is_some() {
                continue;
            }
            entries.push((ch, rows));
        }

        if entries.is_empty() {
            return Err(format!(
                "No glyphs found in monogram metrics {metrics_path}"
            ));
        }

        entries.sort_by_key(|(ch, _)| *ch);

        let line_height = entries
            .iter()
            .map(|(_, rows)| rows.len() as u32)
            .max()
            .unwrap_or(12)
            .max(1);
        let max_bits = entries
            .iter()
            .flat_map(|(_, rows)| rows.iter())
            .map(|v| 32_u32.saturating_sub(v.leading_zeros()))
            .max()
            .unwrap_or(5)
            .max(1);

        let cell_w = max_bits + 1; // 1px breathing room
        let cell_h = line_height;
        let cols = 32_u32;
        let rows_count = ((entries.len() as u32).saturating_add(cols - 1)) / cols;
        let atlas_w = (cols * cell_w).max(1);
        let atlas_h = (rows_count * cell_h).max(1);

        let mut image = Image::gen_image_color(
            atlas_w as u16,
            atlas_h as u16,
            Color::new(0.0, 0.0, 0.0, 0.0),
        );
        let mut glyphs = HashMap::new();

        for (i, (ch, rows)) in entries.iter().enumerate() {
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;
            let ox = col * cell_w;
            let oy = row * cell_h;

            let glyph_w_bits = rows
                .iter()
                .map(|v| 32_u32.saturating_sub(v.leading_zeros()))
                .max()
                .unwrap_or(1)
                .max(1);
            let glyph_h = (rows.len() as u32).max(1);

            for (py, row_bits) in rows.iter().enumerate() {
                let y = oy + py as u32;
                for px in 0..glyph_w_bits {
                    if (row_bits & (1 << px)) != 0 {
                        image.set_pixel(ox + px, y, WHITE);
                    }
                }
            }

            let advance = if *ch == ' ' { 4.0 } else { 6.0 };
            glyphs.insert(
                *ch,
                GlyphMetrics {
                    src: Rect::new(ox as f32, oy as f32, glyph_w_bits as f32, glyph_h as f32),
                    x_offset: 0.0,
                    y_offset: 0.0,
                    x_advance: advance,
                },
            );
        }

        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        let fallback = if glyphs.contains_key(&'?') { '?' } else { ' ' };

        Ok(Self {
            texture,
            glyphs,
            fallback,
            line_height: line_height as f32,
        })
    }

    fn glyph_for(&self, ch: char) -> Option<&GlyphMetrics> {
        self.glyphs
            .get(&ch)
            .or_else(|| self.glyphs.get(&self.fallback))
    }

    pub fn measure(&self, text: &str, scale: i32, spacing: i32) -> Vec2 {
        let scale = scale.max(1) as f32;
        let spacing = spacing.max(0) as f32;
        let mut max_w = 0.0_f32;
        let mut line_w = 0.0_f32;
        let mut lines = 1_u32;

        for ch in text.chars() {
            if ch == '\n' {
                max_w = max_w.max(line_w);
                line_w = 0.0;
                lines += 1;
                continue;
            }

            if let Some(g) = self.glyph_for(ch) {
                line_w += g.x_advance * scale + spacing;
            }
        }
        max_w = max_w.max(line_w);

        Vec2::new(max_w.max(0.0), lines as f32 * self.line_height * scale)
    }

    pub fn draw(&self, text: &str, x: f32, y: f32, scale: i32, color: Color, spacing: i32) {
        let scale = scale.max(1) as f32;
        let spacing = spacing.max(0) as f32;
        let mut pen_x = x;
        let mut pen_y = y;

        for ch in text.chars() {
            if ch == '\n' {
                pen_x = x;
                pen_y += self.line_height * scale;
                continue;
            }

            if let Some(g) = self.glyph_for(ch) {
                draw_texture_ex(
                    &self.texture,
                    pen_x + g.x_offset * scale,
                    pen_y + g.y_offset * scale,
                    color,
                    DrawTextureParams {
                        source: Some(g.src),
                        dest_size: Some(vec2(g.src.w * scale, g.src.h * scale)),
                        ..Default::default()
                    },
                );
                pen_x += g.x_advance * scale + spacing;
            }
        }
    }
}
