// All drawing methods: world, HUD, orbs, overlays, background.
// super::GameState, macroquad, crate::config, crate::enemy, crate::orb
use crate::config::{
    BOUNDARY_X, Biome, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, SCREEN_H, SCREEN_W, TOP_BORDER_BOTTOM,
    TOP_BORDER_TOP, TOP_UPGRADE_LANE_BOTTOM, TOP_UPGRADE_LANE_TOP, UPGRADE_LANE_BOTTOM,
    UPGRADE_LANE_TOP,
};
use crate::drone::RemoteDroneLane;
use crate::enemy::{EnemyKind, EnemyState};
use crate::orb::{OrbPhase, OrbType};
use macroquad::prelude::*;

use super::{EXPLOSION_FRAME_COUNT, EXPLOSION_FRAME_DUR, GameState, PAUSE_BTN_X};

impl GameState {
    pub fn draw(&mut self) {
        self.draw_background();
        if self.shields.count() > 0 {
            let shield_x = BOUNDARY_X - 3.0 + self.shields.shake.offset_x();
            let shield_y = ENEMY_LANE_TOP as f32;
            let shield_h = (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32;
            if self.shields.has_explosive() {
                let orange = Color::from_rgba(255, 140, 0, 255);
                self.boundary_shield_sprite.draw_3slice_vertical_hsl(
                    shield_x,
                    shield_y,
                    shield_h,
                    "top",
                    "mid",
                    "bot",
                    orange,
                    &self.color_blend_material,
                );
            } else {
                self.boundary_shield_sprite
                    .draw_3slice_vertical(shield_x, shield_y, shield_h, "top", "mid", "bot", WHITE);
            }
        }
        self.draw_orbs();
        self.player_sprite
            .draw(self.player.x + self.player.shake.offset_x(), self.player.y);
        for (i, drone) in self.drones.iter().enumerate() {
            if i == 1 {
                self.drone_sprite.draw_flipped_y(drone.x, drone.y);
            } else {
                self.drone_sprite.draw(drone.x, drone.y);
            }
        }
        for rd in &self.remote_drones {
            if rd.lane == RemoteDroneLane::Top {
                self.drone_remote_sprite.draw_flipped_y(rd.x, rd.y);
            } else {
                self.drone_remote_sprite.draw(rd.x, rd.y);
            }
        }
        for p in &self.projectiles {
            self.shot_sprite.draw(p.x, p.y - 0.5);
        }
        for e in &self.enemies {
            let sprite = match e.kind {
                EnemyKind::Small => &self.enemy_small_sprite,
                EnemyKind::Medium => &self.enemy_medium_sprite,
                EnemyKind::Heavy => &self.enemy_heavy_sprite,
                EnemyKind::Large => &self.enemy_large_sprite,
                EnemyKind::XL => &self.enemy_xl_sprite,
                EnemyKind::Boss1 => &self.enemy_boss_1_sprite,
            };
            let tint = if e.state == EnemyState::Breaching {
                e.windup_tint()
            } else {
                WHITE
            };
            let draw_x = e.x + e.shake.offset_x();
            sprite.draw_tinted(draw_x, e.y, tint);
            if e.kind == EnemyKind::Boss1 {
                self.enemy_boss_1_sprite
                    .draw_tinted_row(draw_x, e.y, WHITE, 1);
            }
            let flash_color = e.flash.tint();
            if flash_color != WHITE {
                sprite.draw_additive(draw_x, e.y, flash_color, 0.7, &self.additive_material);
            }
        }

        let explosion_positions: Vec<(f32, f32, u32)> = self
            .explosions
            .iter()
            .map(|exp| {
                let frame =
                    ((exp.timer / EXPLOSION_FRAME_DUR) as u32).min(EXPLOSION_FRAME_COUNT - 1);
                (exp.x, exp.y, frame)
            })
            .collect();
        for (x, y, frame) in explosion_positions {
            self.explosion_sprite.draw_frame(x, y, frame, WHITE);
        }

        self.draw_shield_hud();
        self.draw_upgrade_hud();
        self.draw_biome_hud();
        self.draw_run_timer_hud();
        self.draw_pause_button();
        self.draw_floating_texts();

        let flash = self.screen_flash.tint();
        if flash != WHITE {
            draw_rectangle(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32, flash);
        }

        if self.game_over {
            self.draw_game_over();
        }

        if self.at_title || self.paused {
            if let Some(event_name) = self.event_placeholder {
                self.draw_event_placeholder(event_name);
            } else {
                self.draw_title_pause_screen();
            }
        }

        if self.at_shop {
            self.draw_shop();
        }
    }

    fn draw_orbs(&mut self) {
        let draw_list: Vec<(f32, f32, OrbType, Color, bool, f32, f32)> = self
            .orbs
            .iter()
            .map(|o| {
                let inactive = o.phase == OrbPhase::Inactive;
                let tint = if inactive {
                    Color::from_rgba(180, 180, 180, 160)
                } else {
                    WHITE
                };
                (
                    o.x,
                    o.y,
                    o.orb_type,
                    tint,
                    inactive,
                    o.activation_progress,
                    o.decay_blink_timer,
                )
            })
            .collect();
        for (sx, sy, orb_type, tint, inactive, activation_progress, decay_blink_timer) in draw_list
        {
            let sprite = match orb_type {
                OrbType::Burst => &mut self.orb_sprite_burst,
                OrbType::Damage => &mut self.orb_sprite_damage,
                OrbType::Shield => &mut self.orb_sprite_shield,
                OrbType::Drone => &mut self.orb_sprite_drone,
                OrbType::DroneRemote => &mut self.orb_sprite_drone_remote,
                OrbType::Explosive => &mut self.orb_sprite_explosive,
                OrbType::FireRate => &mut self.orb_sprite_fire_rate,
                OrbType::Pierce => &mut self.orb_sprite_pierce,
                OrbType::Stagger => &mut self.orb_sprite_stagger,
            };
            if inactive {
                sprite.draw_tinted_frozen(sx, sy, WHITE);
                draw_rectangle(sx, sy, 20.0, 20.0, Color::from_rgba(0, 0, 0, 130));
                if activation_progress < 1.0 {
                    let frame = (activation_progress * 4.0).floor().clamp(0.0, 3.0) as u32;
                    if frame < 3 {
                        self.orb_sprite_seal.draw_frame(sx, sy, frame + 1, WHITE);
                    }
                    let blink_visible = if activation_progress == 0.0 {
                        true
                    } else {
                        let delay = crate::config::SEAL_BLINK_DELAY;
                        let rate = crate::config::SEAL_BLINK_RATE;
                        decay_blink_timer < delay
                            || (((decay_blink_timer - delay) / rate) as u32).is_multiple_of(2)
                    };
                    if blink_visible {
                        self.orb_sprite_seal.draw_frame(sx, sy, frame, WHITE);
                    }
                }
            } else {
                sprite.draw_tinted(sx, sy, tint);
            }
        }
    }

    fn draw_game_over(&self) {
        let overlay = Color::from_rgba(0, 0, 0, 160);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, 180.0, overlay);
        let title = "GAME OVER";
        let time_line = format!("TIME {}", self.format_run_timer());
        let kb_line = format!(
            "KILLS {}  BREACHES {}",
            self.kills_total, self.breaches_total
        );

        let title_size = self.logo_font.measure(title, 1, 1);
        let title_x = (SCREEN_W as f32 - title_size.x) * 0.5;
        self.logo_font.draw(
            title,
            title_x,
            38.0,
            1,
            Color::from_rgba(220, 30, 30, 255),
            1,
        );

        let time_size = self.ui_font.measure(&time_line, 1, 1);
        let time_x = (SCREEN_W as f32 - time_size.x) * 0.5;
        self.ui_font.draw(
            &time_line,
            time_x,
            64.0,
            1,
            Color::from_rgba(245, 245, 245, 255),
            1,
        );

        let kb_size = self.ui_font.measure(&kb_line, 1, 1);
        let kb_x = (SCREEN_W as f32 - kb_size.x) * 0.5;
        self.ui_font.draw(
            &kb_line,
            kb_x,
            78.0,
            1,
            Color::from_rgba(220, 220, 220, 255),
            1,
        );

        let earned_line = format!("+{} META", self.meta_points_earned);
        let earned_size = self.ui_font.measure(&earned_line, 1, 1);
        let earned_x = (SCREEN_W as f32 - earned_size.x) * 0.5;
        self.ui_font.draw(
            &earned_line,
            earned_x,
            96.0,
            1,
            Color::from_rgba(79, 217, 195, 255),
            1,
        );

        let total_line = format!("TOTAL  {}", self.save.meta_points);
        let total_size = self.ui_font.measure(&total_line, 1, 1);
        let total_x = (SCREEN_W as f32 - total_size.x) * 0.5;
        self.ui_font.draw(
            &total_line,
            total_x,
            110.0,
            1,
            Color::from_rgba(150, 150, 150, 255),
            1,
        );

        let sel_col = Color::from_rgba(240, 240, 180, 255);
        let unsel_col = Color::from_rgba(140, 140, 140, 255);
        let menu_items = ["RUN AGAIN", "MAIN MENU"];
        for (i, item) in menu_items.iter().enumerate() {
            let selected = i == self.game_over_cursor;
            let prefix = if selected { "> " } else { "  " };
            let line = format!("{prefix}{item}");
            let col = if selected { sel_col } else { unsel_col };
            let sz = self.ui_font.measure(&line, 1, 1);
            let lx = (SCREEN_W as f32 - sz.x) * 0.5;
            self.ui_font
                .draw(&line, lx, 130.0 + i as f32 * 14.0, 1, col, 1);
        }
    }

    fn draw_title_pause_screen(&self) {
        let overlay = Color::from_rgba(0, 0, 0, 200);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, 180.0, overlay);

        let logo_y = 28.0_f32;
        let logo_x = (SCREEN_W as f32 - self.logo_sprite.tile_w as f32) * 0.5;
        self.logo_sprite.draw(logo_x, logo_y);

        let subtitle = "LAST CONVOY DEFENSE";
        let sub_sz = self.ui_font.measure(subtitle, 1, 1);
        let sub_x = (SCREEN_W as f32 - sub_sz.x) * 0.5;
        let sub_y = logo_y + self.logo_sprite.tile_h as f32 + 5.0;
        self.ui_font.draw(
            subtitle,
            sub_x,
            sub_y,
            1,
            Color::from_rgba(200, 200, 200, 255),
            1,
        );

        let controls: &[(&str, &str)] = &[
            ("UP / DOWN", "MOVE"),
            ("COLLECT ORB", "PASS THROUGH ORB"),
            ("PAUSE", "P  /  ESC  /  TAP"),
        ];
        let label_col = Color::from_rgba(180, 220, 255, 255);
        let value_col = Color::from_rgba(230, 230, 230, 255);
        let mut y = 78.0_f32;
        for (label, value) in controls {
            let lsz = self.ui_font.measure(label, 1, 1);
            let mid = SCREEN_W as f32 * 0.5;
            self.ui_font
                .draw(label, mid - lsz.x - 4.0, y, 1, label_col, 1);
            self.ui_font.draw(value, mid + 4.0, y, 1, value_col, 1);
            y += 13.0;
        }

        if self.paused {
            let prompt = "P / ESC / TAP  RESUME";
            let psz = self.ui_font.measure(prompt, 1, 1);
            let px = (SCREEN_W as f32 - psz.x) * 0.5;
            self.ui_font.draw(
                prompt,
                px,
                155.0,
                1,
                Color::from_rgba(240, 240, 180, 255),
                1,
            );
        } else {
            let sel_col = Color::from_rgba(240, 240, 180, 255);
            let unsel_col = Color::from_rgba(180, 180, 180, 255);
            let items = ["PLAY", "UPGRADES"];
            for (i, item) in items.iter().enumerate() {
                let prefix = if i == self.title_cursor { "> " } else { "  " };
                let line = format!("{prefix}{item}");
                let col = if i == self.title_cursor {
                    sel_col
                } else {
                    unsel_col
                };
                let sz = self.ui_font.measure(&line, 1, 1);
                let lx = (SCREEN_W as f32 - sz.x) * 0.5;
                self.ui_font
                    .draw(&line, lx, 148.0 + i as f32 * 13.0, 1, col, 1);
            }
        }
    }

    fn draw_event_placeholder(&self, name: &str) {
        let overlay = Color::from_rgba(0, 0, 0, 200);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32, overlay);

        let label = "PLACEHOLDER FOR EVENT...";
        let lsz = self.ui_font.measure(label, 1, 1);
        let lx = (SCREEN_W as f32 - lsz.x) * 0.5;
        self.ui_font
            .draw(label, lx, 70.0, 1, Color::from_rgba(200, 200, 200, 255), 1);

        let nsz = self.ui_font.measure(name, 1, 1);
        let nx = (SCREEN_W as f32 - nsz.x) * 0.5;
        self.ui_font
            .draw(name, nx, 86.0, 1, Color::from_rgba(240, 240, 180, 255), 1);

        let prompt = if self.event_placeholder_timer > 0.0 {
            format!("WAIT {}...", self.event_placeholder_timer.ceil() as u32)
        } else {
            "P / ESC / TAP  CONTINUE".to_string()
        };
        let psz = self.ui_font.measure(&prompt, 1, 1);
        let px = (SCREEN_W as f32 - psz.x) * 0.5;
        self.ui_font.draw(
            &prompt,
            px,
            110.0,
            1,
            Color::from_rgba(180, 220, 255, 255),
            1,
        );
    }

    fn draw_biome_hud(&self) {
        let biome_num = match self.current_biome {
            Biome::InfectedAtmosphere => 1,
            Biome::LowOrbit => 2,
            Biome::OuterSystem => 3,
            Biome::DeepSpace => 4,
        };
        let label = format!("BIOME {biome_num}");
        let sz = self.ui_font.measure(&label, 1, 1);
        let x = (SCREEN_W as f32 - sz.x) * 0.5;
        self.ui_font
            .draw(&label, x, 7.0, 1, Color::from_rgba(220, 220, 220, 255), 1);
    }

    fn draw_shield_hud(&self) {
        let size = 8.0_f32;
        let height = 7.0_f32;
        let gap = 2.0_f32;
        let start_x = 5.0_f32;
        let y = 7.0_f32;
        let shield_cap = self.biome_shield_cap();
        let dark = Color::from_rgba(40, 40, 40, 255);
        for i in 0..shield_cap {
            let x = start_x + i as f32 * (size + gap);
            draw_rectangle(x, y, size, height, dark);
        }
        let n_normal = self
            .shields
            .segments
            .iter()
            .filter(|s| !s.explosive)
            .count();
        let has_explosive = self.shields.has_explosive();
        for i in 0..n_normal {
            let x = start_x + i as f32 * (size + gap);
            draw_rectangle(x, y, size, height, Color::from_rgba(0, 200, 80, 255));
        }
        if has_explosive {
            let x = start_x + n_normal as f32 * (size + gap);
            draw_rectangle(x, y, size, height, Color::from_rgba(255, 140, 0, 255));
        }
    }

    fn draw_upgrade_hud(&mut self) {
        let shield_cap = self.biome_shield_cap();
        let shield_area_end = 5.0 + shield_cap as f32 * (8.0 + 2.0) + 2.0;
        let icon_size = 10.0_f32;
        let icon_gap = 2.0_f32;
        let y = 5.0_f32;
        let mut x = shield_area_end;
        let bar_dark = Color::from_rgba(40, 40, 40, 255);
        let teal_fill = Color::from_rgba(79, 217, 195, 255);

        {
            let ratio = if self.config.buff_damage_duration > 0.0 {
                (self.damage_buff_t / self.config.buff_damage_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.damage_buff_active() {
                self.orb_sprite_damage
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(x + icon_size, y, 2.0, icon_size, bar_dark);
                draw_rectangle(
                    x + icon_size,
                    y + icon_size * (1.0 - ratio),
                    2.0,
                    icon_size * ratio,
                    teal_fill,
                );
                x += icon_size + 2.0 + icon_gap;
            }
        }
        {
            let ratio = if self.config.buff_fire_rate_duration > 0.0 {
                (self.fire_rate_buff_t / self.config.buff_fire_rate_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.fire_rate_buff_active() {
                self.orb_sprite_fire_rate
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(x + icon_size, y, 2.0, icon_size, bar_dark);
                draw_rectangle(
                    x + icon_size,
                    y + icon_size * (1.0 - ratio),
                    2.0,
                    icon_size * ratio,
                    teal_fill,
                );
                x += icon_size + 2.0 + icon_gap;
            }
        }
        {
            let ratio = if self.config.buff_burst_duration > 0.0 {
                (self.burst_buff_t / self.config.buff_burst_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.burst_buff_active() {
                self.orb_sprite_burst
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(x + icon_size, y, 2.0, icon_size, bar_dark);
                draw_rectangle(
                    x + icon_size,
                    y + icon_size * (1.0 - ratio),
                    2.0,
                    icon_size * ratio,
                    teal_fill,
                );
                x += icon_size + 2.0 + icon_gap;
            }
        }
        {
            let ratio = if self.config.buff_pierce_duration > 0.0 {
                (self.pierce_buff_t / self.config.buff_pierce_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.pierce_buff_active() {
                self.orb_sprite_pierce
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(x + icon_size, y, 2.0, icon_size, bar_dark);
                draw_rectangle(
                    x + icon_size,
                    y + icon_size * (1.0 - ratio),
                    2.0,
                    icon_size * ratio,
                    teal_fill,
                );
                x += icon_size + 2.0 + icon_gap;
            }
        }
        {
            let ratio = if self.config.buff_stagger_duration > 0.0 {
                (self.stagger_buff_t / self.config.buff_stagger_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.stagger_buff_active() {
                self.orb_sprite_stagger
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(x + icon_size, y, 2.0, icon_size, bar_dark);
                draw_rectangle(
                    x + icon_size,
                    y + icon_size * (1.0 - ratio),
                    2.0,
                    icon_size * ratio,
                    teal_fill,
                );
                #[allow(unused_assignments)]
                {
                    x += icon_size + 2.0 + icon_gap;
                }
            }
        }
    }

    fn draw_run_timer_hud(&self) {
        let timer = self.format_run_timer();
        let size = self.ui_font.measure(&timer, 1, 1);
        let x = PAUSE_BTN_X - 4.0 - size.x;
        let y = ((TOP_BORDER_BOTTOM - TOP_BORDER_TOP + 1) as f32 - size.y) * 0.5 + 2.0;
        self.ui_font
            .draw(&timer, x, y, 1, Color::from_rgba(220, 220, 220, 255), 1);
    }

    fn draw_pause_button(&self) {
        let bx = PAUSE_BTN_X + 1.0;
        let by = 6.0;
        let bar_h = 10.0;
        let col = Color::from_rgba(180, 180, 180, 200);
        draw_rectangle(bx, by, 3.0, bar_h, col);
        draw_rectangle(bx + 5.0, by, 3.0, bar_h, col);
    }

    fn draw_floating_texts(&self) {
        for t in &self.floating_texts {
            let alpha = (t.life / t.ttl).clamp(0.0, 1.0);
            let mut color = t.color;
            color.a *= alpha;
            self.monogram_font.draw(&t.text, t.x, t.y, 1, color, 1);
        }
    }

    fn draw_background(&self) {
        let w = SCREEN_W as f32;

        let space_very_dark = Color::from_rgba(10, 14, 22, 255);
        let teal_very_dark = Color::from_rgba(0, 50, 44, 255);

        let tb_h = (TOP_BORDER_BOTTOM - TOP_BORDER_TOP + 1) as f32;
        self.top_bar_sprite.draw_9slice(0.0, 0.0, w, tb_h, WHITE);

        draw_rectangle(
            0.0,
            TOP_UPGRADE_LANE_TOP as f32,
            w,
            (TOP_UPGRADE_LANE_BOTTOM - TOP_UPGRADE_LANE_TOP + 1) as f32,
            teal_very_dark,
        );

        draw_rectangle(
            0.0,
            ENEMY_LANE_TOP as f32,
            w,
            (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32,
            space_very_dark,
        );
        draw_rectangle(
            0.0,
            UPGRADE_LANE_TOP as f32,
            w,
            (UPGRADE_LANE_BOTTOM - UPGRADE_LANE_TOP + 1) as f32,
            teal_very_dark,
        );

        {
            let lane_top = ENEMY_LANE_TOP as f32;
            let lane_h = (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32;
            if self.current_biome == Biome::InfectedAtmosphere {
                let layer_w = 320.0_f32;
                let time_left = (self.biome_duration() - self.biome_time).max(0.0);
                let frame_ys: [f32; 7] = [0.0, 371.0, 742.0, 1113.0, 1484.0, 1855.0, 2226.0];
                let vert_offset_max = frame_ys[1] - SCREEN_H as f32 + ENEMY_LANE_TOP as f32;
                let vert_offset = if time_left > 5.0 {
                    vert_offset_max
                } else {
                    let t = (5.0 - time_left) / 5.0;
                    let ease = (1.0 - t) * (1.0 - t);
                    vert_offset_max * ease
                };

                draw_texture_ex(
                    &self.city_biome_texture,
                    0.0,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[0] + vert_offset, layer_w, lane_h)),
                        dest_size: Some(vec2(layer_w, lane_h)),
                        ..Default::default()
                    },
                );

                draw_texture_ex(
                    &self.city_biome_texture,
                    0.0,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[1] + vert_offset, layer_w, lane_h)),
                        dest_size: Some(vec2(layer_w, lane_h)),
                        ..Default::default()
                    },
                );

                let wrapped = self.city_bg_scroll_offsets[0].rem_euclid(layer_w);
                let src_bb = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[2] + vert_offset, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.city_biome_texture,
                    wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_bb.clone(),
                );
                draw_texture_ex(&self.city_biome_texture, wrapped, lane_top, WHITE, src_bb);

                let wrapped = self.city_bg_scroll_offsets[1].rem_euclid(layer_w);
                let src_bm = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[3] + vert_offset, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.city_biome_texture,
                    wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_bm.clone(),
                );
                draw_texture_ex(&self.city_biome_texture, wrapped, lane_top, WHITE, src_bm);

                let wrapped = self.city_bg_scroll_offsets[2].rem_euclid(layer_w);
                let src_fb = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[4] + vert_offset, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.city_biome_texture,
                    wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_fb.clone(),
                );
                draw_texture_ex(&self.city_biome_texture, wrapped, lane_top, WHITE, src_fb);

                let wrapped = self.city_bg_scroll_offsets[3].rem_euclid(layer_w);
                let src_st = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[5] + vert_offset, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.city_biome_texture,
                    wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_st.clone(),
                );
                draw_texture_ex(&self.city_biome_texture, wrapped, lane_top, WHITE, src_st);

                draw_texture_ex(
                    &self.city_biome_texture,
                    0.0,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[6] + vert_offset, layer_w, lane_h)),
                        dest_size: Some(vec2(layer_w, lane_h)),
                        ..Default::default()
                    },
                );
            } else if self.current_biome == Biome::LowOrbit {
                let layer_w = 320.0_f32;
                let clip_y = ENEMY_LANE_TOP as f32;
                let frame_ys: [f32; 4] = [0.0, 180.0, 360.0, 540.0];

                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    0.0,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[0] + clip_y, layer_w, lane_h)),
                        dest_size: Some(vec2(layer_w, lane_h)),
                        ..Default::default()
                    },
                );

                let wrapped = self.outer_system_scroll_offsets[1].rem_euclid(layer_w);
                let src_stars = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[1] + clip_y, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_stars.clone(),
                );
                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    wrapped,
                    lane_top,
                    WHITE,
                    src_stars,
                );

                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    0.0,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[2] + clip_y, layer_w, lane_h)),
                        dest_size: Some(vec2(layer_w, lane_h)),
                        ..Default::default()
                    },
                );

                let moon_wrapped = self.low_atmo_moon_offset.rem_euclid(layer_w);
                let src_moon = DrawTextureParams {
                    source: Some(Rect::new(0.0, frame_ys[3] + clip_y, layer_w, lane_h)),
                    dest_size: Some(vec2(layer_w, lane_h)),
                    ..Default::default()
                };
                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    moon_wrapped - layer_w,
                    lane_top,
                    WHITE,
                    src_moon.clone(),
                );
                draw_texture_ex(
                    &self.low_atmosphere_texture,
                    moon_wrapped,
                    lane_top,
                    WHITE,
                    src_moon,
                );
            } else if self.current_biome == Biome::OuterSystem {
                let layer_h = 160.0_f32;
                let clip_y = (layer_h - lane_h) / 2.0;
                let tex_w = 284.0_f32;
                for (i, &offset) in self.outer_system_scroll_offsets.iter().enumerate() {
                    let src_y = i as f32 * layer_h + clip_y;
                    let src = DrawTextureParams {
                        source: Some(Rect::new(0.0, src_y, tex_w, lane_h)),
                        dest_size: Some(vec2(tex_w, lane_h)),
                        ..Default::default()
                    };
                    if i == 0 {
                        draw_texture_ex(
                            &self.outer_system_texture,
                            BOUNDARY_X,
                            lane_top,
                            WHITE,
                            src,
                        );
                    } else {
                        let speeds = [
                            self.config.bg_parallax_speed_back,
                            self.config.bg_parallax_speed_stars,
                            self.config.bg_parallax_speed_props,
                        ];
                        if speeds[i] == 0.0 {
                            continue;
                        }
                        let wrapped = offset.rem_euclid(tex_w);
                        draw_texture_ex(
                            &self.outer_system_texture,
                            BOUNDARY_X + wrapped - tex_w,
                            lane_top,
                            WHITE,
                            src.clone(),
                        );
                        draw_texture_ex(
                            &self.outer_system_texture,
                            BOUNDARY_X + wrapped,
                            lane_top,
                            WHITE,
                            src,
                        );
                    }
                }
            } else {
                let tex_w = 284.0_f32;
                let lane_h_ds = 115.0_f32;
                let frame_ys: [f32; 9] =
                    [0.0, 115.0, 230.0, 345.0, 460.0, 575.0, 690.0, 805.0, 920.0];

                draw_texture_ex(
                    &self.deep_space_texture,
                    BOUNDARY_X,
                    lane_top,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[0], tex_w, lane_h_ds)),
                        dest_size: Some(vec2(tex_w, lane_h_ds)),
                        ..Default::default()
                    },
                );

                for &frame_y in &frame_ys[1..=3] {
                    let wrapped = self.deep_space_scroll_offsets[0].rem_euclid(tex_w);
                    let src = DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_y, tex_w, lane_h_ds)),
                        dest_size: Some(vec2(tex_w, lane_h_ds)),
                        ..Default::default()
                    };
                    draw_texture_ex(
                        &self.deep_space_texture,
                        BOUNDARY_X + wrapped - tex_w,
                        lane_top,
                        WHITE,
                        src.clone(),
                    );
                    draw_texture_ex(
                        &self.deep_space_texture,
                        BOUNDARY_X + wrapped,
                        lane_top,
                        WHITE,
                        src,
                    );
                }

                {
                    let wrapped = self.deep_space_scroll_offsets[1].rem_euclid(tex_w);
                    let src = DrawTextureParams {
                        source: Some(Rect::new(0.0, frame_ys[4], tex_w, lane_h_ds)),
                        dest_size: Some(vec2(tex_w, lane_h_ds)),
                        ..Default::default()
                    };
                    draw_texture_ex(
                        &self.deep_space_texture,
                        BOUNDARY_X + wrapped - tex_w,
                        lane_top,
                        WHITE,
                        src.clone(),
                    );
                    draw_texture_ex(
                        &self.deep_space_texture,
                        BOUNDARY_X + wrapped,
                        lane_top,
                        WHITE,
                        src,
                    );
                }

                {
                    let x_shift = (self.deep_space_scroll_offsets[2] * 0.4).sin() * 2.0;
                    for &frame_y in &frame_ys[5..] {
                        draw_texture_ex(
                            &self.deep_space_texture,
                            BOUNDARY_X + x_shift,
                            lane_top,
                            WHITE,
                            DrawTextureParams {
                                source: Some(Rect::new(0.0, frame_y, tex_w, lane_h_ds)),
                                dest_size: Some(vec2(tex_w, lane_h_ds)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        let rail_x = 0.0_f32;
        let tile_h = 36.0_f32;
        let rail_start = TOP_UPGRADE_LANE_TOP as f32;
        let rail_end = UPGRADE_LANE_BOTTOM as f32 + 1.0;
        let mut cursor = rail_start;
        while cursor < rail_end {
            let available = rail_end - cursor;
            if available >= tile_h {
                self.rail_wall_sprite.draw(rail_x, cursor);
            } else {
                self.rail_wall_sprite
                    .draw_clipped_h(rail_x, cursor, available);
            }
            cursor += tile_h;
        }

        let track_h = 21.0_f32;
        let top_track_y = TOP_UPGRADE_LANE_TOP as f32;
        let bot_track_y = UPGRADE_LANE_BOTTOM as f32 + 1.0 - track_h;
        self.upgrade_track_sprite.draw_front_tiled_h(
            0.0,
            top_track_y,
            SCREEN_W as f32,
            "front",
            "rail",
            true,
        );
        self.upgrade_track_sprite.draw_front_tiled_h(
            0.0,
            bot_track_y,
            SCREEN_W as f32,
            "front",
            "rail",
            false,
        );
    }
}
