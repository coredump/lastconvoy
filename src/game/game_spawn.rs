// Enemy spawn logic: tick_spawn, try_place_enemy, coverage computation.
// super::GameState, crate::config, crate::enemy
use crate::config::{
    BIG_INJECT_BASE_INTERVAL, Biome, COVERAGE_HYSTERESIS, COVERAGE_ZONE_LEFT, COVERAGE_ZONE_RIGHT,
    COVERAGE_ZONE_WIDTH, Config, ENEMY_HEAVY_H, ENEMY_HEAVY_HP, ENEMY_HEAVY_SPEED, ENEMY_HEAVY_W,
    ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, ENEMY_LARGE_H, ENEMY_LARGE_HP, ENEMY_LARGE_SPEED,
    ENEMY_LARGE_W, ENEMY_MEDIUM_H, ENEMY_MEDIUM_HP, ENEMY_MEDIUM_SPEED, ENEMY_MEDIUM_W,
    ENEMY_SMALL_H, ENEMY_SMALL_HP, ENEMY_SMALL_SPEED, ENEMY_SMALL_W, ENEMY_XL_H, ENEMY_XL_W,
    SCREEN_W, SHIELDED_FREQ_SCALE, SPAWN_LEAD_PX, SPAWN_MAX_RETRIES, SPAWN_SLOT_COUNT,
    SPAWN_SLOT_WIDTH,
};
use crate::enemy::{Enemy, EnemyKind};
use macroquad::prelude::rand;

use super::{GameState, aabb_overlap};

impl GameState {
    pub(super) fn tick_spawn(&mut self) {
        self.spawn_ctrl.inject_timer -= crate::config::SPAWN_TICK_INTERVAL;

        let ramp_dur = self.config.spawn_ramp_duration;
        if ramp_dur > 0.0 && self.run_time < ramp_dur {
            self.spawn_ctrl.ramp_log_timer -= crate::config::SPAWN_TICK_INTERVAL;
            if self.spawn_ctrl.ramp_log_timer <= 0.0 {
                self.spawn_ctrl.ramp_log_timer = ramp_dur / 2.0;
                let ramp_factor = (self.run_time / ramp_dur).clamp(0.0, 1.0);
                let coverage = compute_coverage(&self.enemies);
                let target = coverage_target(self.run_time, &self.config);
                if let Some(log) = &mut self.debug_log {
                    log.log(
                        self.run_time,
                        &format!(
                            "spawn ramp: {:.0}% of ramp_duration, target={:.2}, coverage={:.2}",
                            ramp_factor * 100.0,
                            target,
                            coverage,
                        ),
                    );
                }
            }
        }
        let coverage = compute_coverage(&self.enemies);
        let target = coverage_target(self.run_time, &self.config);
        let target = if self.current_biome == Biome::InfectedAtmosphere {
            let cycle_pos = self.run_time % self.config.biome1_lull_interval;
            if cycle_pos < self.config.biome1_lull_duration {
                target * self.config.biome1_lull_intensity
            } else {
                target
            }
        } else {
            target
        };

        let inject_coverage_cap = (target + 0.10).min(1.0);
        if self.spawn_ctrl.inject_timer <= 0.0 && coverage < inject_coverage_cap {
            let kind = if let Some(forced) = self.config.debug_force_enemy {
                forced
            } else if self.config.debug_all_enemies {
                let r = rand::gen_range(0usize, 4);
                [
                    EnemyKind::Medium,
                    EnemyKind::Heavy,
                    EnemyKind::Large,
                    EnemyKind::Small,
                ][r]
            } else {
                match self.current_biome {
                    Biome::InfectedAtmosphere => {
                        if self.biome_time >= self.config.biome_1_medium_delay {
                            EnemyKind::Medium
                        } else {
                            EnemyKind::Small
                        }
                    }
                    Biome::LowOrbit => {
                        let r = rand::gen_range(0usize, 2);
                        [EnemyKind::Medium, EnemyKind::Heavy][r]
                    }
                    Biome::OuterSystem => {
                        let r = rand::gen_range(0usize, 3);
                        [EnemyKind::Medium, EnemyKind::Heavy, EnemyKind::Large][r]
                    }
                    Biome::DeepSpace => {
                        let r = rand::gen_range(0usize, 4);
                        [
                            EnemyKind::Medium,
                            EnemyKind::Heavy,
                            EnemyKind::Large,
                            EnemyKind::XL,
                        ][r]
                    }
                }
            };
            let late_pressure = (self.run_time / 600.0).clamp(0.0, 1.0);
            let interval_scale = 1.0 - 0.25 * late_pressure;
            let jitter = rand::gen_range(-0.3_f32, 0.3);
            self.spawn_ctrl.inject_timer = BIG_INJECT_BASE_INTERVAL * interval_scale + jitter;
            self.try_place_enemy(kind);
        } else if coverage < target - COVERAGE_HYSTERESIS {
            let kind = self.config.debug_force_enemy.unwrap_or(EnemyKind::Small);
            self.try_place_enemy(kind);
        }
    }

    pub(super) fn try_place_enemy(&mut self, kind: EnemyKind) {
        let (w, h, hp, base_speed) = match kind {
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
            EnemyKind::XL => (
                ENEMY_XL_W,
                ENEMY_XL_H,
                self.config.xl_hp,
                self.config.xl_speed,
            ),
        };

        let windup_time = match kind {
            EnemyKind::Small => self.config.windup_time_small,
            EnemyKind::Medium => self.config.windup_time_medium,
            EnemyKind::Heavy => self.config.windup_time_heavy,
            EnemyKind::Large => self.config.windup_time_large,
            EnemyKind::XL => self.config.windup_time_xl,
        };

        let kind_weight = match kind {
            EnemyKind::Heavy => self.config.hp_scale_heavy_mult,
            EnemyKind::Large => self.config.hp_scale_large_mult,
            _ => 1.0,
        };
        let loop_scale = 1.0 + self.loop_count as f32 * self.config.biome_loop_hp_mult;
        let hp_mult = (1.0 + self.config.enemy_hp_scale * self.run_time * kind_weight) * loop_scale;
        let hp = ((hp as f32) * hp_mult).round().max(1.0) as i32;

        let y_min = ENEMY_LANE_TOP as f32;
        let y_max = (ENEMY_LANE_BOTTOM as f32 - h).max(y_min);

        for _ in 0..SPAWN_MAX_RETRIES {
            let advance = rand::gen_range(1usize, 3);
            self.spawn_ctrl.cursor = (self.spawn_ctrl.cursor + advance) % SPAWN_SLOT_COUNT;

            let phase_drift = self.spawn_ctrl.cursor as f32 * SPAWN_SLOT_WIDTH;
            let x = SCREEN_W as f32 + SPAWN_LEAD_PX + phase_drift;

            let base_y = y_min + rand::gen_range(0.0_f32, 1.0) * (y_max - y_min);
            let jitter_y = rand::gen_range(-2.0_f32, 2.0);
            let y = (base_y + jitter_y).clamp(y_min, y_max);

            let speed_jitter = rand::gen_range(0.97_f32, 1.03);
            let speed_mult = (1.0 + self.config.speed_scale_per_sec * self.run_time)
                .min(self.config.speed_scale_cap);
            let speed = base_speed * speed_mult * speed_jitter;

            let overlap = self.enemies.iter().any(|e| {
                (e.x - x).abs() < 64.0 && aabb_overlap(x, y, w, h, e.x, e.y, e.width, e.height)
            });
            if overlap {
                continue;
            }

            let mut enemy = Enemy::new(x, y, kind, hp, speed, w, h, windup_time);
            let shield_chance = (SHIELDED_FREQ_SCALE * self.run_time).min(0.5);
            if rand::gen_range(0.0_f32, 1.0) < shield_chance {
                enemy.shielded = true;
                enemy.shield_hp = 1;
            }
            let shielded = enemy.shielded;
            self.enemies.push(enemy);
            self.dlog(&format!(
                "ENEMY_SPAWN kind={:?} shielded={}",
                kind, shielded
            ));
            return;
        }
    }
}

pub(super) fn compute_coverage(enemies: &[crate::enemy::Enemy]) -> f32 {
    let mut intervals: Vec<(f32, f32)> = enemies
        .iter()
        .filter_map(|e| {
            let left = e.x.max(COVERAGE_ZONE_LEFT);
            let right = (e.x + e.width).min(COVERAGE_ZONE_RIGHT);
            if right > left {
                Some((left, right))
            } else {
                None
            }
        })
        .collect();

    if intervals.is_empty() {
        return 0.0;
    }

    intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut merged_len = 0.0_f32;
    let mut cur_left = intervals[0].0;
    let mut cur_right = intervals[0].1;
    for (l, r) in intervals.iter().skip(1) {
        if *l <= cur_right {
            cur_right = cur_right.max(*r);
        } else {
            merged_len += cur_right - cur_left;
            cur_left = *l;
            cur_right = *r;
        }
    }
    merged_len += cur_right - cur_left;
    merged_len / COVERAGE_ZONE_WIDTH
}

pub(super) fn coverage_target(run_time: f32, cfg: &Config) -> f32 {
    let t = (run_time / 720.0).min(1.0);
    let full_target = 0.72 + t * 0.18;
    if cfg.spawn_ramp_duration <= 0.0 {
        return full_target;
    }
    let ramp = (run_time / cfg.spawn_ramp_duration).clamp(0.0, 1.0);
    cfg.spawn_ramp_start_coverage + (full_target - cfg.spawn_ramp_start_coverage) * ramp
}
