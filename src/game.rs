use macroquad::prelude::*;

use crate::boundary::Boundary;
use crate::config::{
    BOTTOM_BORDER_BOTTOM, BOTTOM_BORDER_TOP, BOUNDARY_X, Config, DIVIDER_BOTTOM, DIVIDER_TOP,
    ENEMY_HEAVY_H, ENEMY_HEAVY_HP, ENEMY_HEAVY_SPEED, ENEMY_HEAVY_W, ENEMY_LANE_BOTTOM,
    ENEMY_LANE_TOP, ENEMY_LARGE_H, ENEMY_LARGE_HP, ENEMY_LARGE_SPEED, ENEMY_LARGE_W,
    ENEMY_MEDIUM_H, ENEMY_MEDIUM_HP, ENEMY_MEDIUM_SPEED, ENEMY_MEDIUM_W, ENEMY_SMALL_H,
    ENEMY_SMALL_HP, ENEMY_SMALL_SPEED, ENEMY_SMALL_W, HEAVY_INTRO_TIME, LARGE_INTRO_TIME,
    MEDIUM_INTRO_TIME, PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_X, PROJECTILE_H, PROJECTILE_W, SCREEN_W,
    SHIELDED_FREQ_SCALE, SPAWN_RATE_SCALE, TOP_BORDER_BOTTOM, TOP_BORDER_TOP, UPGRADE_LANE_BOTTOM,
    UPGRADE_LANE_TOP,
};
use crate::drone::Drone;
use crate::elite::EliteEvent;
use crate::enemy::{Enemy, EnemyKind};
use crate::input::InputState;
use crate::orb::Orb;
use crate::player::Player;
use crate::projectile::{Projectile, ProjectileSource};
use crate::shield::ShieldSegment;
use crate::sprite::SpriteSheet;

pub struct GameState {
    pub config: Config,
    pub player: Player,
    pub player_sprite: SpriteSheet,
    pub enemy_small_sprite: SpriteSheet,
    pub enemy_medium_sprite: SpriteSheet,
    pub enemy_heavy_sprite: SpriteSheet,
    pub enemy_large_sprite: SpriteSheet,
    pub shields: Vec<ShieldSegment>,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub orbs: Vec<Orb>,
    pub drones: Vec<Drone>,
    pub boundary: Boundary,
    pub elite_event: EliteEvent,
    pub input: InputState,
    pub enemy_spawn_timer: f32,
    pub orb_spawn_timer: f32,
    pub elite_timer: f32,
    pub miniboss_timer: f32,
    pub run_time: f32,
}

impl GameState {
    pub fn new(
        config: Config,
        player_sprite: SpriteSheet,
        enemy_small_sprite: SpriteSheet,
        enemy_medium_sprite: SpriteSheet,
        enemy_heavy_sprite: SpriteSheet,
        enemy_large_sprite: SpriteSheet,
    ) -> Self {
        let player_y = ((ENEMY_LANE_TOP + ENEMY_LANE_BOTTOM) / 2) as f32;
        let player = Player::new(
            PLAYER_X,
            player_y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            config.player_speed,
            config.player_fire_rate,
        );

        let shields = (0..config.player_starting_shields)
            .map(|_| ShieldSegment::new(true))
            .collect();

        let boundary = Boundary::new(config.boundary_slot_count);

        Self {
            player,
            player_sprite,
            enemy_small_sprite,
            enemy_medium_sprite,
            enemy_heavy_sprite,
            enemy_large_sprite,
            shields,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            orbs: Vec::new(),
            drones: Vec::new(),
            boundary,
            elite_event: EliteEvent::new(),
            input: InputState::new(),
            enemy_spawn_timer: 0.0,
            orb_spawn_timer: 0.0,
            elite_timer: config.elite_interval,
            miniboss_timer: config.miniboss_interval,
            run_time: 0.0,
            config,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.run_time += dt;
        self.input.update(&self.config);

        // Player movement
        let axis = self.input.axis;
        self.player.update(axis, dt);

        // Auto-fire
        if self.player.should_fire() {
            let proj_x = self.player.x + self.player.width;
            let proj_y = self.player.y + (self.player.height - PROJECTILE_H) / 2.0;
            self.projectiles.push(Projectile::new(
                proj_x,
                proj_y,
                self.config.projectile_speed,
                ProjectileSource::Player,
            ));
        }

        // Update projectiles
        for p in &mut self.projectiles {
            p.update(dt);
        }

        // Projectile-enemy collision (player projectiles only; drone shots never hit enemies per spec)
        for p in &mut self.projectiles {
            if !p.alive || p.source != ProjectileSource::Player {
                continue;
            }
            for e in &mut self.enemies {
                if e.is_dead() {
                    continue;
                }
                if aabb_overlap(
                    p.x,
                    p.y,
                    PROJECTILE_W,
                    PROJECTILE_H,
                    e.x,
                    e.y,
                    e.width,
                    e.height,
                ) {
                    e.take_damage(1);
                    p.alive = false;
                    break;
                }
            }
        }

        self.projectiles.retain(|p| !p.should_remove());
        self.enemies.retain(|e| !e.is_dead());

        // Enemy spawning
        let effective_interval =
            self.config.enemy_spawn_interval / (1.0 + SPAWN_RATE_SCALE * self.run_time);
        self.enemy_spawn_timer -= dt;
        if self.enemy_spawn_timer <= 0.0 {
            self.enemy_spawn_timer = effective_interval;
            self.spawn_enemy();
        }

        // Update enemies
        for e in &mut self.enemies {
            e.update(dt);
            // Boundary arrival
            if !e.at_boundary && e.x <= BOUNDARY_X {
                match e.kind {
                    EnemyKind::Small => {
                        // Small enemies despawn on boundary arrival (damage handled in P1.6)
                        e.hp = 0;
                    }
                    _ => {
                        e.at_boundary = true;
                        e.x = BOUNDARY_X;
                    }
                }
            }
        }
        self.enemies.retain(|e| !e.is_off_screen() && !e.is_dead());

        // Resolve enemy stacking: sort stably by x ascending, then for each enemy find
        // the nearest blocker ahead of it in the same y-band and clamp behind it.
        self.enemies.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        for i in 1..self.enemies.len() {
            let (front, back) = self.enemies.split_at_mut(i);
            let follower = &mut back[0];
            // Scan right-to-left through all enemies ahead to find the nearest y-overlapping one.
            for blocker in front.iter().rev() {
                let v_overlap = follower.y < blocker.y + blocker.height
                    && follower.y + follower.height > blocker.y;
                if v_overlap {
                    let right_edge = blocker.x + blocker.width;
                    if follower.x < right_edge {
                        follower.x = right_edge;
                    }
                    break; // list is sorted; first vertical-overlap match from the right is the blocker
                }
            }
        }

        // Advance sprite animations
        self.player_sprite.update(dt);
        self.enemy_small_sprite.update(dt);
        self.enemy_medium_sprite.update(dt);
        self.enemy_heavy_sprite.update(dt);
        self.enemy_large_sprite.update(dt);
    }

    pub fn draw(&self) {
        self.draw_background();
        self.player_sprite.draw(self.player.x, self.player.y);
        for p in &self.projectiles {
            p.draw();
        }
        for e in &self.enemies {
            let sprite = match e.kind {
                EnemyKind::Small => &self.enemy_small_sprite,
                EnemyKind::Medium => &self.enemy_medium_sprite,
                EnemyKind::Heavy => &self.enemy_heavy_sprite,
                EnemyKind::Large => &self.enemy_large_sprite,
            };
            sprite.draw(e.x, e.y);
        }
    }

    fn spawn_enemy(&mut self) {
        let kind = self.pick_enemy_kind();
        let (w, h, hp, speed) = match kind {
            EnemyKind::Small => (
                ENEMY_SMALL_W,
                ENEMY_SMALL_H,
                ENEMY_SMALL_HP,
                ENEMY_SMALL_SPEED,
            ),
            EnemyKind::Medium => (
                ENEMY_MEDIUM_W,
                ENEMY_MEDIUM_H,
                ENEMY_MEDIUM_HP,
                ENEMY_MEDIUM_SPEED,
            ),
            EnemyKind::Heavy => (
                ENEMY_HEAVY_W,
                ENEMY_HEAVY_H,
                ENEMY_HEAVY_HP,
                ENEMY_HEAVY_SPEED,
            ),
            EnemyKind::Large => (
                ENEMY_LARGE_W,
                ENEMY_LARGE_H,
                ENEMY_LARGE_HP,
                ENEMY_LARGE_SPEED,
            ),
        };
        let y_min = ENEMY_LANE_TOP as f32;
        let y_max = (ENEMY_LANE_BOTTOM as f32 - h).max(y_min);
        let y = y_min + rand::gen_range(0.0_f32, 1.0) * (y_max - y_min);
        let mut enemy = Enemy::new(SCREEN_W as f32, y, kind, hp, speed, w, h);

        // Shielded chance increases with run time, capped at 50%
        let shield_chance = (SHIELDED_FREQ_SCALE * self.run_time).min(0.5);
        if rand::gen_range(0.0_f32, 1.0) < shield_chance {
            enemy.shielded = true;
            enemy.shield_hp = 1;
        }

        self.enemies.push(enemy);
    }

    fn pick_enemy_kind(&self) -> EnemyKind {
        let mut pool: &[EnemyKind] = &[EnemyKind::Small];
        if self.run_time >= LARGE_INTRO_TIME {
            pool = &[
                EnemyKind::Small,
                EnemyKind::Medium,
                EnemyKind::Heavy,
                EnemyKind::Large,
            ];
        } else if self.run_time >= HEAVY_INTRO_TIME {
            pool = &[EnemyKind::Small, EnemyKind::Medium, EnemyKind::Heavy];
        } else if self.run_time >= MEDIUM_INTRO_TIME {
            pool = &[EnemyKind::Small, EnemyKind::Medium];
        }
        let idx = rand::gen_range(0, pool.len());
        pool[idx]
    }

    fn draw_background(&self) {
        let w = SCREEN_W as f32;

        // Palette colors (from lcdss_palette.gpl)
        let space_very_dark = Color::from_rgba(10, 14, 22, 255);
        let steel_dark = Color::from_rgba(30, 38, 51, 255);
        let steel_mid = Color::from_rgba(58, 70, 90, 255);
        let teal_very_dark = Color::from_rgba(0, 50, 44, 255);

        // Top border (rows 0–15): Steel Dark fill + Steel Mid accent on inner edge
        let tb_y = TOP_BORDER_TOP as f32;
        let tb_h = (TOP_BORDER_BOTTOM - TOP_BORDER_TOP + 1) as f32;
        draw_rectangle(0.0, tb_y, w, tb_h, steel_dark);
        draw_rectangle(0.0, tb_y + tb_h - 1.0, w, 1.0, steel_mid);

        // Enemy lane (rows 16–119): Space Very Dark
        draw_rectangle(
            0.0,
            ENEMY_LANE_TOP as f32,
            w,
            (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32,
            space_very_dark,
        );

        // Divider (rows 120–123): Steel Mid flat placeholder
        draw_rectangle(
            0.0,
            DIVIDER_TOP as f32,
            w,
            (DIVIDER_BOTTOM - DIVIDER_TOP + 1) as f32,
            steel_mid,
        );

        // Upgrade lane (rows 124–163): Upgrade Teal Very Dark
        draw_rectangle(
            0.0,
            UPGRADE_LANE_TOP as f32,
            w,
            (UPGRADE_LANE_BOTTOM - UPGRADE_LANE_TOP + 1) as f32,
            teal_very_dark,
        );

        // Bottom border (rows 164–179): Steel Dark fill + Steel Mid accent on inner edge
        let bb_y = BOTTOM_BORDER_TOP as f32;
        let bb_h = (BOTTOM_BORDER_BOTTOM - BOTTOM_BORDER_TOP + 1) as f32;
        draw_rectangle(0.0, bb_y, w, bb_h, steel_dark);
        draw_rectangle(0.0, bb_y, w, 1.0, steel_mid);
    }
}

#[allow(clippy::too_many_arguments)]
fn aabb_overlap(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}
