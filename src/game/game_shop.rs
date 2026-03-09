// Permanent upgrade shop: navigation, purchase logic, and drawing.
// super::GameState, crate::save, macroquad
use crate::config::{SCREEN_H, SCREEN_W};
use macroquad::prelude::*;

use super::GameState;

const ITEM_H: f32 = 18.0;
const LIST_TOP: f32 = 52.0;

impl GameState {
    pub(super) fn update_shop(&mut self, dt: f32) {
        self.shop_flash_timer = (self.shop_flash_timer - dt).max(0.0);

        let catalog_len = self.upgrade_catalog.upgrade.len();
        if catalog_len == 0 {
            if is_key_pressed(KeyCode::Escape) {
                self.at_shop = false;
            }
            return;
        }

        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.shop_cursor = (self.shop_cursor + 1) % catalog_len;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.shop_cursor = (self.shop_cursor + catalog_len - 1) % catalog_len;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.at_shop = false;
            return;
        }
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            self.try_purchase();
        }

        if let Some((tx, ty)) = self.input.touch_tapped_pos {
            let item_area_top = LIST_TOP;
            let item_area_bottom = LIST_TOP + catalog_len as f32 * ITEM_H;
            if ty >= item_area_top && ty < item_area_bottom {
                let idx = ((ty - item_area_top) / ITEM_H) as usize;
                if idx < catalog_len {
                    if self.shop_cursor == idx {
                        self.try_purchase();
                    } else {
                        self.shop_cursor = idx;
                    }
                }
            } else if ty < item_area_top {
                self.at_shop = false;
            }
            let _ = tx;
        }
    }

    fn try_purchase(&mut self) {
        let def = match self.upgrade_catalog.upgrade.get(self.shop_cursor) {
            Some(d) => d.clone(),
            None => return,
        };
        let current_level = self.save.permanent_upgrades.get_level(&def.id);
        if current_level >= def.max_level {
            return;
        }
        let cost = def.cost_per_level[current_level as usize];
        if self.save.meta_points < cost {
            return;
        }
        self.save.meta_points -= cost;
        self.save
            .permanent_upgrades
            .set_level(&def.id, current_level + 1);
        crate::save::write_save(&self.save);
        self.shop_flash_timer = 0.4;
    }

    pub(super) fn draw_shop(&self) {
        let overlay = Color::from_rgba(0, 0, 10, 210);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, SCREEN_H as f32, overlay);

        let title = "UPGRADES";
        let title_sz = self.logo_font.measure(title, 1, 1);
        let title_x = (SCREEN_W as f32 - title_sz.x) * 0.5;
        self.logo_font.draw(
            title,
            title_x,
            8.0,
            1,
            Color::from_rgba(240, 240, 180, 255),
            1,
        );

        let pts_label = format!("META  {}", self.save.meta_points);
        let pts_sz = self.ui_font.measure(&pts_label, 1, 1);
        let pts_x = (SCREEN_W as f32 - pts_sz.x) * 0.5;
        self.ui_font.draw(
            &pts_label,
            pts_x,
            30.0,
            1,
            Color::from_rgba(180, 220, 255, 255),
            1,
        );

        let hint = "UP/DN NAVIGATE  SPACE/ENTER BUY  ESC BACK";
        let hint_sz = self.ui_font.measure(hint, 1, 1);
        let hint_x = (SCREEN_W as f32 - hint_sz.x) * 0.5;
        self.ui_font.draw(
            hint,
            hint_x,
            SCREEN_H as f32 - 12.0,
            1,
            Color::from_rgba(120, 120, 120, 255),
            1,
        );

        let catalog = &self.upgrade_catalog.upgrade;
        for (i, def) in catalog.iter().enumerate() {
            let y = LIST_TOP + i as f32 * ITEM_H;
            let selected = i == self.shop_cursor;
            let current_level = self.save.permanent_upgrades.get_level(&def.id);
            let is_maxed = current_level >= def.max_level;

            let row_bg = if selected {
                Color::from_rgba(40, 40, 60, 180)
            } else {
                Color::from_rgba(0, 0, 0, 0)
            };
            draw_rectangle(0.0, y, SCREEN_W as f32, ITEM_H, row_bg);

            let cursor_marker = if selected { ">" } else { " " };
            self.ui_font.draw(
                cursor_marker,
                4.0,
                y + 2.0,
                1,
                Color::from_rgba(240, 240, 180, 255),
                1,
            );

            let name_col = if is_maxed {
                Color::from_rgba(120, 120, 120, 255)
            } else if selected {
                Color::from_rgba(240, 240, 180, 255)
            } else {
                Color::from_rgba(220, 220, 220, 255)
            };
            self.ui_font
                .draw(&def.display_name, 14.0, y + 2.0, 1, name_col, 1);

            let pip_x_start = 130.0_f32;
            let pip_w = 7.0_f32;
            let pip_h = 5.0_f32;
            let pip_gap = 2.0_f32;
            let pip_y = y + 6.0;
            for p in 0..def.max_level {
                let px = pip_x_start + p as f32 * (pip_w + pip_gap);
                let filled = p < current_level;
                let pip_col = if filled {
                    Color::from_rgba(79, 217, 195, 255)
                } else {
                    Color::from_rgba(50, 50, 50, 255)
                };
                draw_rectangle(px, pip_y, pip_w, pip_h, pip_col);
            }

            let cost_str = if is_maxed {
                "MAX".to_string()
            } else {
                format!("{}", def.cost_per_level[current_level as usize])
            };
            let cost_col = if is_maxed {
                Color::from_rgba(100, 100, 100, 255)
            } else if self.save.meta_points >= def.cost_per_level[current_level as usize] {
                Color::from_rgba(220, 220, 100, 255)
            } else {
                Color::from_rgba(160, 80, 80, 255)
            };
            let cost_sz = self.ui_font.measure(&cost_str, 1, 1);
            self.ui_font.draw(
                &cost_str,
                SCREEN_W as f32 - cost_sz.x - 6.0,
                y + 2.0,
                1,
                cost_col,
                1,
            );
        }

        if self.shop_flash_timer > 0.0 {
            let alpha = ((self.shop_flash_timer * 4.0) as u8).min(80);
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W as f32,
                SCREEN_H as f32,
                Color::from_rgba(79, 217, 195, alpha),
            );
        }
    }
}
