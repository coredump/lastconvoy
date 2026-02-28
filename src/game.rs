use macroquad::prelude::*;

use crate::boundary::Boundary;
use crate::config::SHAKE_DURATION;
use crate::config::SHAKE_INTENSITY;
use crate::config::{
    BIG_INJECT_BASE_INTERVAL, BOTTOM_BORDER_BOTTOM, BOTTOM_BORDER_TOP, BOUNDARY_X,
    COVERAGE_HYSTERESIS, COVERAGE_ZONE_LEFT, COVERAGE_ZONE_RIGHT, COVERAGE_ZONE_WIDTH, Config,
    DIVIDER_BOTTOM, DIVIDER_TOP, ENEMY_ELITE_H, ENEMY_ELITE_W, ENEMY_HEAVY_H, ENEMY_HEAVY_HP,
    ENEMY_HEAVY_SPEED, ENEMY_HEAVY_W, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, ENEMY_LARGE_H,
    ENEMY_LARGE_HP, ENEMY_LARGE_SPEED, ENEMY_LARGE_W, ENEMY_MEDIUM_H, ENEMY_MEDIUM_HP,
    ENEMY_MEDIUM_SPEED, ENEMY_MEDIUM_W, ENEMY_SMALL_H, ENEMY_SMALL_HP, ENEMY_SMALL_SPEED,
    ENEMY_SMALL_W, HEAVY_INTRO_TIME, LARGE_INTRO_TIME, MAX_BURST_LEVEL, MAX_DAMAGE_LEVEL,
    MAX_FIRE_RATE_LEVEL, MAX_PIERCE_LEVEL, MAX_STAGGER_LEVEL, MEDIUM_INTRO_TIME, ORB_H, ORB_W,
    PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_X, PROJECTILE_H, PROJECTILE_W, SCREEN_W,
    SHIELDED_FREQ_SCALE, SPAWN_LEAD_PX, SPAWN_MAX_RETRIES, SPAWN_SLOT_COUNT, SPAWN_SLOT_WIDTH,
    SPAWN_TICK_INTERVAL, STAGGER_DURATIONS, STAGGER_KNOCKBACK_SPEED, TOP_BORDER_BOTTOM,
    TOP_BORDER_TOP, UPGRADE_LANE_BOTTOM, UPGRADE_LANE_TOP,
};
use crate::drone::Drone;
use crate::elite::EliteEvent;
use crate::enemy::{Enemy, EnemyKind};
use crate::input::InputState;
use crate::orb::Orb;
use crate::orb::{OrbPhase, OrbType};
use crate::player::Player;
use crate::projectile::{Projectile, ProjectileSource};
use crate::shield::ShieldSystem;
use crate::sprite::Sprite;

pub struct SpawnController {
    pub tick_accum: f32,
    pub cursor: usize,
    pub inject_timer: f32,
    pub ramp_log_timer: f32,
}

impl SpawnController {
    fn new() -> Self {
        Self {
            tick_accum: 0.0,
            cursor: 0,
            inject_timer: BIG_INJECT_BASE_INTERVAL,
            ramp_log_timer: 0.0,
        }
    }

    fn reset(&mut self) {
        self.tick_accum = 0.0;
        self.cursor = 0;
        self.inject_timer = BIG_INJECT_BASE_INTERVAL;
        self.ramp_log_timer = 0.0;
    }
}

pub struct GameState {
    pub config: Config,
    pub player: Player,
    pub player_sprite: Sprite,
    pub enemy_small_sprite: Sprite,
    pub enemy_medium_sprite: Sprite,
    pub enemy_heavy_sprite: Sprite,
    pub enemy_large_sprite: Sprite,
    pub enemy_elite_sprite: Sprite,
    pub boundary_shield_sprite: Sprite,
    pub shields: ShieldSystem,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub orbs: Vec<Orb>,
    pub orb_sprite_damage: Sprite,
    pub orb_sprite_defense: Sprite,
    pub orb_sprite_drone: Sprite,
    pub orb_sprite_fire_rate: Sprite,
    pub orb_sprite_burst: Sprite,
    pub orb_sprite_pierce: Sprite,
    pub orb_sprite_stagger: Sprite,
    pub damage_level: usize,
    pub fire_rate_level: usize,
    pub burst_level: usize,
    pub pierce_level: usize,
    pub stagger_level: usize,
    pub burst_timer: f32,
    pub burst_ready: bool,
    pub drones: Vec<Drone>,
    pub boundary: Boundary,
    pub elite_event: EliteEvent,
    pub input: InputState,
    pub spawn_ctrl: SpawnController,
    pub orb_spawn_timer: f32,
    pub elite_timer: f32,
    pub miniboss_timer: f32,
    pub run_time: f32,
    pub game_over: bool,
    pub debug_log: Option<crate::debug_log::DebugLog>,
}

impl GameState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Config,
        player_sprite: Sprite,
        enemy_small_sprite: Sprite,
        enemy_medium_sprite: Sprite,
        enemy_heavy_sprite: Sprite,
        enemy_large_sprite: Sprite,
        enemy_elite_sprite: Sprite,
        boundary_shield_sprite: Sprite,
        orb_sprite_damage: Sprite,
        orb_sprite_defense: Sprite,
        orb_sprite_drone: Sprite,
        orb_sprite_fire_rate: Sprite,
        orb_sprite_burst: Sprite,
        orb_sprite_pierce: Sprite,
        orb_sprite_stagger: Sprite,
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

        let boundary = Boundary::new(config.boundary_slot_count);

        Self {
            player,
            player_sprite,
            enemy_small_sprite,
            enemy_medium_sprite,
            enemy_heavy_sprite,
            enemy_large_sprite,
            enemy_elite_sprite,
            boundary_shield_sprite,
            shields: ShieldSystem::new(config.player_starting_shields),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            orbs: Vec::new(),
            orb_sprite_damage,
            orb_sprite_defense,
            orb_sprite_drone,
            orb_sprite_fire_rate,
            orb_sprite_burst,
            orb_sprite_pierce,
            orb_sprite_stagger,
            damage_level: 0,
            fire_rate_level: 0,
            burst_level: 0,
            pierce_level: 0,
            stagger_level: 0,
            burst_timer: 0.0,
            burst_ready: false,
            drones: Vec::new(),
            boundary,
            elite_event: EliteEvent::new(),
            input: InputState::new(),
            spawn_ctrl: SpawnController::new(),
            orb_spawn_timer: 0.0,
            elite_timer: config.elite_interval,
            miniboss_timer: config.miniboss_interval,
            run_time: 0.0,
            game_over: false,
            debug_log: if config.debug_log_gameplay {
                Some(crate::debug_log::DebugLog::new(
                    &config.debug_log_file.clone(),
                ))
            } else {
                None
            },
            config,
        }
    }

    /// Reset all mutable game state for a new run (keeps config and sprites).
    pub fn reset(&mut self) {
        let player_y = ((ENEMY_LANE_TOP + ENEMY_LANE_BOTTOM) / 2) as f32;
        self.player = Player::new(
            PLAYER_X,
            player_y,
            PLAYER_WIDTH,
            PLAYER_HEIGHT,
            self.config.player_speed,
            self.config.player_fire_rate,
        );
        self.shields = ShieldSystem::new(self.config.player_starting_shields);
        self.boundary = Boundary::new(self.config.boundary_slot_count);
        self.enemies.clear();
        self.projectiles.clear();
        self.orbs.clear();
        self.drones.clear();
        self.damage_level = 0;
        self.fire_rate_level = 0;
        self.player.fire_rate = self.config.fire_rate_levels[0];
        self.burst_level = 0;
        self.pierce_level = 0;
        self.stagger_level = 0;
        self.burst_timer = 0.0;
        self.burst_ready = false;
        self.elite_event = EliteEvent::new();
        self.spawn_ctrl.reset();
        self.orb_spawn_timer = 0.0;
        self.elite_timer = self.config.elite_interval;
        self.miniboss_timer = self.config.miniboss_interval;
        self.run_time = 0.0;
        self.game_over = false;
    }

    fn dlog(&mut self, msg: &str) {
        if let Some(log) = &mut self.debug_log {
            log.log(self.run_time, msg);
        }
    }

    /// Deal one damage event to the player: consume one shield segment, or die if none remain.
    fn take_player_damage(&mut self) {
        self.player.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
        if !self.shields.take_hit() {
            self.game_over = true;
        }
        let remaining = self.shields.count();
        self.dlog(&format!(
            "PLAYER_DMG count=1 shields_remaining={}",
            remaining
        ));
    }

    pub fn update(&mut self, dt: f32) {
        // On death: wait for any key then restart.
        if self.game_over {
            if is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::R)
            {
                self.reset();
            }
            return;
        }

        self.run_time += dt;
        self.input.update(&self.config);

        // Player movement
        let axis = self.input.axis;
        self.player.update(axis, dt);

        // Burst timer
        if self.burst_level > 0 {
            self.burst_timer -= dt;
            if self.burst_timer <= 0.0 {
                self.burst_ready = true;
                self.burst_timer =
                    self.config.burst_intervals[self.burst_level.min(MAX_BURST_LEVEL) - 1];
            }
        }

        // Auto-fire
        if self.player.should_fire() {
            let proj_x = self.player.x + self.player.width;
            let proj_y = self.player.y + (self.player.height - PROJECTILE_H) / 2.0;
            let is_burst = self.burst_ready;
            if is_burst {
                self.burst_ready = false;
            }
            self.projectiles.push(Projectile::new(
                proj_x,
                proj_y,
                self.config.projectile_speed,
                ProjectileSource::Player,
                is_burst,
                self.pierce_level as i32,
            ));
        }

        // Update projectiles
        for p in &mut self.projectiles {
            p.update(dt);
        }

        // Projectile-enemy collision (player projectiles only; drone shots never hit enemies per spec)
        let mut kill_logs: Vec<String> = Vec::new();
        let base_dmg = self.config.damage_levels[self.damage_level.min(MAX_DAMAGE_LEVEL - 1)];
        for p in &mut self.projectiles {
            if !p.alive || p.source != ProjectileSource::Player {
                continue;
            }
            let player_dmg = if p.is_burst {
                base_dmg * self.config.burst_damage_multiplier
            } else {
                base_dmg
            };
            for e in &mut self.enemies {
                if e.is_dead() || p.hit_enemies.contains(&e.id) {
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
                    p.hit_enemies.push(e.id);
                    e.take_damage(player_dmg);
                    if !e.is_dead()
                        && self.stagger_level > 0
                        && matches!(
                            e.kind,
                            EnemyKind::Small | EnemyKind::Medium | EnemyKind::Heavy
                        )
                    {
                        let duration =
                            STAGGER_DURATIONS[self.stagger_level.min(MAX_STAGGER_LEVEL) - 1];
                        e.apply_knockback(duration);
                    }
                    if e.is_dead() {
                        let dps = if e.shots_taken > 0 {
                            e.damage_taken as f32 / e.shots_taken as f32
                        } else {
                            0.0
                        };
                        kill_logs.push(format!(
                            "KILL {:?} hp_max={} speed={:.1} shielded={} shield_hp={} dmg_total={} shots={} dmg_per_shot={:.2}",
                            e.kind, e.max_hp, e.speed, e.shielded, e.shield_hp.max(0),
                            e.damage_taken, e.shots_taken, dps
                        ));
                    }
                    if p.pierce_remaining <= 0 {
                        p.alive = false;
                        break;
                    }
                    p.pierce_remaining -= 1;
                }
            }
        }
        for msg in kill_logs {
            self.dlog(&msg);
        }

        self.projectiles.retain(|p| !p.should_remove());

        // Release boundary slots for dead enemies before clearing them.
        for e in &self.enemies {
            if e.is_dead()
                && let Some(slot) = e.slot_id
            {
                self.boundary.release_slot(slot);
            }
        }

        // Promote queued enemies (at boundary, no slot) to any newly freed slots.
        // Nearest to the boundary (smallest x) gets priority.
        if self.boundary.has_free_slot() {
            let mut queued: Vec<usize> = self
                .enemies
                .iter()
                .enumerate()
                .filter(|(_, e)| e.at_boundary && e.slot_id.is_none() && !e.is_dead())
                .map(|(i, _)| i)
                .collect();
            queued.sort_by(|&a, &b| self.enemies[a].x.partial_cmp(&self.enemies[b].x).unwrap());
            for idx in queued {
                if let Some(slot) = self.boundary.occupy_slot() {
                    self.enemies[idx].slot_id = Some(slot);
                    self.enemies[idx].x = BOUNDARY_X;
                } else {
                    break;
                }
            }
        }

        self.enemies.retain(|e| !e.is_dead() || e.shake.is_active());

        // Coverage-based enemy spawning
        self.spawn_ctrl.tick_accum += dt;
        while self.spawn_ctrl.tick_accum >= SPAWN_TICK_INTERVAL {
            self.spawn_ctrl.tick_accum -= SPAWN_TICK_INTERVAL;
            self.tick_spawn();
        }

        // Update enemies and handle boundary arrival/damage.
        // Collect damage events first to avoid borrow conflict with take_player_damage.
        let mut damage_events: u32 = 0;
        let mut boundary_logs: Vec<String> = Vec::new();

        for e in &mut self.enemies {
            e.update(dt, STAGGER_KNOCKBACK_SPEED);

            if !e.at_boundary && e.knockback_timer <= 0.0 && e.x <= BOUNDARY_X {
                match e.kind {
                    EnemyKind::Small => {
                        // 1 damage event then despawn.
                        damage_events += 1;
                        e.hp = 0;
                        boundary_logs.push("BREACH Small".to_string());
                    }
                    _ => {
                        e.at_boundary = true;
                        e.x = BOUNDARY_X;
                        // Try to occupy a boundary slot.
                        if let Some(slot) = self.boundary.occupy_slot() {
                            e.slot_id = Some(slot);
                        }
                        // If no slot: enemy is queued — stops in place, no damage tick.
                    }
                }
            }

            // Slotted enemies tick damage.
            if e.at_boundary && e.slot_id.is_some() {
                e.damage_timer += dt;
                if e.damage_timer >= self.config.boundary_damage_tick {
                    e.damage_timer = 0.0;
                    damage_events += 1;
                    boundary_logs.push(format!("BOUNDARY_DMG {:?} hp={}", e.kind, e.hp));
                }
            }
        }
        for msg in boundary_logs {
            self.dlog(&msg);
        }

        // Apply collected damage events.
        for _ in 0..damage_events {
            self.take_player_damage();
            if self.game_over {
                break;
            }
        }

        self.enemies
            .retain(|e| !e.is_off_screen() && (!e.is_dead() || e.shake.is_active()));

        // Resolve enemy stacking: sort stably by x ascending, then for each enemy find
        // the nearest blocker ahead of it in the same y-band and clamp behind it.
        self.enemies.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        for i in 1..self.enemies.len() {
            let (front, back) = self.enemies.split_at_mut(i);
            let follower = &mut back[0];
            for blocker in front.iter().rev() {
                let v_overlap = follower.y < blocker.y + blocker.height
                    && follower.y + follower.height > blocker.y;
                if v_overlap {
                    let right_edge = blocker.x + blocker.width;
                    if follower.x < right_edge {
                        follower.x = right_edge;
                    }
                    break;
                }
            }
        }

        // Orb spawning
        self.orb_spawn_timer -= dt;
        if self.orb_spawn_timer <= 0.0 {
            self.orb_spawn_timer = self.config.orb_spawn_interval;
            if self.orbs.len() < self.config.max_active_orbs {
                let lane_mid = (UPGRADE_LANE_TOP + UPGRADE_LANE_BOTTOM + 1) as f32 / 2.0;
                let y = lane_mid - ORB_H / 2.0;
                // Build weighted pool; gate Defense if shields are full.
                let shields_full = self.shields.count() >= crate::shield::MAX_SHIELD_SEGMENTS;
                let mut pool: Vec<OrbType> = Vec::with_capacity(5);
                if self.burst_level < MAX_BURST_LEVEL {
                    pool.push(OrbType::Burst);
                }
                if self.damage_level < MAX_DAMAGE_LEVEL {
                    pool.push(OrbType::Damage);
                }
                if !shields_full {
                    pool.push(OrbType::Defense);
                }
                pool.push(OrbType::Drone);
                if self.fire_rate_level < MAX_FIRE_RATE_LEVEL {
                    pool.push(OrbType::FireRate);
                }
                if self.pierce_level < MAX_PIERCE_LEVEL {
                    pool.push(OrbType::Pierce);
                }
                if self.stagger_level < MAX_STAGGER_LEVEL {
                    pool.push(OrbType::Stagger);
                }
                if let Some(forced) = self.config.debug_force_orb {
                    pool = vec![forced];
                }
                if !pool.is_empty() {
                    let idx = rand::gen_range(0usize, pool.len());
                    let orb_type = pool[idx];
                    self.orbs.push(Orb::new(
                        SCREEN_W as f32,
                        y,
                        ORB_W,
                        ORB_H,
                        self.config.orb_speed,
                        orb_type,
                    ));
                }
            }
        }

        // Projectile-orb collision (player only; drone shots skip)
        for p in &mut self.projectiles {
            if !p.alive || p.source != ProjectileSource::Player {
                continue;
            }
            for o in &mut self.orbs {
                if o.phase == OrbPhase::Inactive
                    && aabb_overlap(
                        p.x,
                        p.y,
                        PROJECTILE_W,
                        PROJECTILE_H,
                        o.x,
                        o.y,
                        o.width,
                        o.height,
                    )
                {
                    o.hit_this_frame = true;
                    p.alive = false;
                    break;
                }
            }
        }

        // Orb movement
        for o in &mut self.orbs {
            o.update(dt);
        }

        // Player-orb collection (active orbs only)
        let mut shield_grants = 0u32;
        let mut damage_collected = 0u32;
        let mut fire_rate_collected = 0u32;
        let mut burst_collected = 0u32;
        let mut pierce_collected = 0u32;
        let mut stagger_collected = 0u32;
        for o in &mut self.orbs {
            if o.phase == OrbPhase::Active
                && aabb_overlap(
                    self.player.x,
                    self.player.y,
                    self.player.width,
                    self.player.height,
                    o.x,
                    o.y,
                    o.width,
                    o.height,
                )
            {
                o.collected = true;
                match o.orb_type {
                    OrbType::Burst => {
                        burst_collected += 1;
                    }
                    OrbType::Defense => {
                        shield_grants += 1;
                    }
                    OrbType::Damage => {
                        damage_collected += 1;
                    }
                    OrbType::FireRate => {
                        fire_rate_collected += 1;
                    }
                    OrbType::Pierce => {
                        pierce_collected += 1;
                    }
                    OrbType::Stagger => {
                        stagger_collected += 1;
                    }
                    OrbType::Drone => {}
                }
            }
        }
        if shield_grants > 0 {
            self.shields.add_segments(shield_grants);
        }
        for _ in 0..damage_collected {
            if self.damage_level < MAX_DAMAGE_LEVEL {
                self.damage_level += 1;
            }
            self.dlog(&format!("ORB_COLLECT Damage level={}", self.damage_level));
        }
        for _ in 0..fire_rate_collected {
            if self.fire_rate_level < MAX_FIRE_RATE_LEVEL {
                self.fire_rate_level += 1;
            }
            let new_rate =
                self.config.fire_rate_levels[self.fire_rate_level.min(MAX_FIRE_RATE_LEVEL - 1)];
            self.player.fire_rate = new_rate;
            self.dlog(&format!(
                "ORB_COLLECT FireRate level={} fire_rate={:.3}",
                self.fire_rate_level, new_rate
            ));
        }
        for _ in 0..burst_collected {
            if self.burst_level < MAX_BURST_LEVEL {
                self.burst_level += 1;
            }
            self.burst_timer =
                self.config.burst_intervals[self.burst_level.min(MAX_BURST_LEVEL) - 1];
            self.dlog(&format!("ORB_COLLECT Burst level={}", self.burst_level));
        }
        for _ in 0..pierce_collected {
            if self.pierce_level < MAX_PIERCE_LEVEL {
                self.pierce_level += 1;
            }
            self.dlog(&format!("ORB_COLLECT Pierce level={}", self.pierce_level));
        }
        for _ in 0..stagger_collected {
            if self.stagger_level < MAX_STAGGER_LEVEL {
                self.stagger_level += 1;
            }
            self.dlog(&format!("ORB_COLLECT Stagger level={}", self.stagger_level));
        }

        let player_cx = self.player.x + self.player.width / 2.0;
        self.orbs.retain(|o| {
            let orb_cx = o.x + o.width / 2.0;
            orb_cx >= player_cx && !o.is_collected()
        });

        // Advance sprite animations
        self.orb_sprite_damage.update(dt);
        self.orb_sprite_defense.update(dt);
        self.orb_sprite_drone.update(dt);
        self.orb_sprite_fire_rate.update(dt);
        self.orb_sprite_burst.update(dt);
        self.orb_sprite_pierce.update(dt);
        self.orb_sprite_stagger.update(dt);
        self.player_sprite.update(dt);
        self.enemy_small_sprite.update(dt);
        self.enemy_medium_sprite.update(dt);
        self.enemy_heavy_sprite.update(dt);
        self.enemy_large_sprite.update(dt);
        self.enemy_elite_sprite.update(dt);
        self.boundary_shield_sprite.update(dt);
        self.shields.update(dt);
    }

    pub fn draw(&mut self) {
        self.draw_background();
        if self.shields.count() > 0 {
            self.boundary_shield_sprite.draw(
                BOUNDARY_X + self.shields.shake.offset_x(),
                ENEMY_LANE_TOP as f32,
            );
        }
        self.player_sprite
            .draw(self.player.x + self.player.shake.offset_x(), self.player.y);
        for p in &self.projectiles {
            p.draw();
        }
        for e in &self.enemies {
            let sprite = match e.kind {
                EnemyKind::Small => &self.enemy_small_sprite,
                EnemyKind::Medium => &self.enemy_medium_sprite,
                EnemyKind::Heavy => &self.enemy_heavy_sprite,
                EnemyKind::Large => &self.enemy_large_sprite,
                EnemyKind::Elite => &self.enemy_elite_sprite,
            };
            sprite.draw(e.x + e.shake.offset_x(), e.y);
        }

        self.draw_orbs();

        self.draw_shield_hud();

        if self.game_over {
            self.draw_game_over();
        }
    }

    fn draw_orbs(&mut self) {
        // Each OrbType has a dedicated Sprite pre-locked to its animation tag,
        // so we never call set_animation() here.
        // Tag indices in upgrades.json: 0=damage, 7=shield, 5=extradrone, 4=stagger
        let draw_list: Vec<(f32, f32, OrbType, Color)> = self
            .orbs
            .iter()
            .map(|o| {
                let tint = if o.phase == OrbPhase::Inactive {
                    let p = o.activation_progress;
                    let v = (100.0 + 155.0 * p) as u8;
                    Color::from_rgba(v, v, v, v)
                } else {
                    WHITE
                };
                (o.x, o.y, o.orb_type, tint)
            })
            .collect();
        for (sx, sy, orb_type, tint) in draw_list {
            let sprite = match orb_type {
                OrbType::Burst => &mut self.orb_sprite_burst,
                OrbType::Damage => &mut self.orb_sprite_damage,
                OrbType::Defense => &mut self.orb_sprite_defense,
                OrbType::Drone => &mut self.orb_sprite_drone,
                OrbType::FireRate => &mut self.orb_sprite_fire_rate,
                OrbType::Pierce => &mut self.orb_sprite_pierce,
                OrbType::Stagger => &mut self.orb_sprite_stagger,
            };
            sprite.draw_tinted(sx, sy, tint);
        }
    }

    fn draw_game_over(&self) {
        // Simple overlay — dim the screen and prompt restart.
        let overlay = Color::from_rgba(0, 0, 0, 160);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, 180.0, overlay);
        // No text rendering in Phase 1 (no font loaded) — the blank screen + shield HUD
        // at 0 segments is sufficient feedback. Restart on Space/Enter/R.
    }

    /// Draw shield segment count in the top-left corner as small green squares.
    fn draw_shield_hud(&self) {
        let count = self.shields.count();
        let size = 4.0_f32;
        let gap = 2.0_f32;
        let start_x = 2.0_f32;
        let y = 2.0_f32;
        for i in 0..count {
            let x = start_x + i as f32 * (size + gap);
            draw_rectangle(x, y, size, size, Color::from_rgba(0, 200, 80, 255));
        }
    }

    fn tick_spawn(&mut self) {
        self.spawn_ctrl.inject_timer -= SPAWN_TICK_INTERVAL;

        // Ramp debug logging: emit a line every ramp_duration/2 seconds while ramping.
        let ramp_dur = self.config.spawn_ramp_duration;
        if ramp_dur > 0.0 && self.run_time < ramp_dur {
            self.spawn_ctrl.ramp_log_timer -= SPAWN_TICK_INTERVAL;
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

        if self.spawn_ctrl.inject_timer <= 0.0 && coverage < target {
            // Inject a big enemy based on run time
            let kind = if self.config.debug_all_enemies || self.run_time >= LARGE_INTRO_TIME {
                let r = rand::gen_range(0usize, 3);
                [EnemyKind::Medium, EnemyKind::Heavy, EnemyKind::Large][r]
            } else if self.run_time >= HEAVY_INTRO_TIME {
                let r = rand::gen_range(0usize, 2);
                [EnemyKind::Medium, EnemyKind::Heavy][r]
            } else if self.run_time >= MEDIUM_INTRO_TIME {
                EnemyKind::Medium
            } else {
                EnemyKind::Small
            };
            // Reset timer with small jitter
            let jitter = rand::gen_range(-0.3_f32, 0.3);
            self.spawn_ctrl.inject_timer = BIG_INJECT_BASE_INTERVAL + jitter;
            self.try_place_enemy(kind);
        } else if coverage < target - COVERAGE_HYSTERESIS {
            self.try_place_enemy(EnemyKind::Small);
        }
    }

    fn try_place_enemy(&mut self, kind: EnemyKind) {
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
            EnemyKind::Elite => (
                ENEMY_ELITE_W,
                ENEMY_ELITE_H,
                self.config.elite_hp,
                self.config.elite_speed,
            ),
        };

        // HP scaling: heavier kinds scale faster
        let kind_weight = match kind {
            EnemyKind::Heavy => self.config.hp_scale_heavy_mult,
            EnemyKind::Large => self.config.hp_scale_large_mult,
            _ => 1.0,
        };
        let hp_mult = 1.0 + self.config.enemy_hp_scale * self.run_time * kind_weight;
        let hp = ((hp as f32) * hp_mult).round().max(1.0) as i32;

        let y_min = ENEMY_LANE_TOP as f32;
        let y_max = (ENEMY_LANE_BOTTOM as f32 - h).max(y_min);

        for _ in 0..SPAWN_MAX_RETRIES {
            // Advance cursor by 1–2 slots
            let advance = rand::gen_range(1usize, 3);
            self.spawn_ctrl.cursor = (self.spawn_ctrl.cursor + advance) % SPAWN_SLOT_COUNT;

            // X: just off the right edge with slot phase drift
            let phase_drift = self.spawn_ctrl.cursor as f32 * SPAWN_SLOT_WIDTH;
            let x = SCREEN_W as f32 + SPAWN_LEAD_PX + phase_drift;

            // Y: lane random with ±2px jitter, clamped
            let base_y = y_min + rand::gen_range(0.0_f32, 1.0) * (y_max - y_min);
            let jitter_y = rand::gen_range(-2.0_f32, 2.0);
            let y = (base_y + jitter_y).clamp(y_min, y_max);

            // Speed: ±3% jitter + time-based scaling
            let speed_jitter = rand::gen_range(0.97_f32, 1.03);
            let speed_mult = (1.0 + self.config.speed_scale_per_sec * self.run_time)
                .min(self.config.speed_scale_cap);
            let speed = base_speed * speed_mult * speed_jitter;

            // AABB overlap check vs enemies within 64px in x
            let overlap = self.enemies.iter().any(|e| {
                (e.x - x).abs() < 64.0 && aabb_overlap(x, y, w, h, e.x, e.y, e.width, e.height)
            });
            if overlap {
                continue;
            }

            // Place enemy
            let mut enemy = Enemy::new(x, y, kind, hp, speed, w, h);
            let shield_chance = (SHIELDED_FREQ_SCALE * self.run_time).min(0.5);
            if rand::gen_range(0.0_f32, 1.0) < shield_chance {
                enemy.shielded = true;
                enemy.shield_hp = 1;
            }
            let spawn_msg = format!(
                "SPAWN {:?} hp={} speed={:.1} shielded={} x={:.1} y={:.1}",
                enemy.kind, enemy.hp, enemy.speed, enemy.shielded, enemy.x, enemy.y
            );
            self.enemies.push(enemy);
            self.dlog(&spawn_msg);
            return;
        }
        // All retries failed — skip this spawn tick
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

fn compute_coverage(enemies: &[Enemy]) -> f32 {
    // Collect x-intervals clamped to the coverage zone [96, 320]
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

    // Sort by left edge and merge overlapping intervals
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

fn coverage_target(run_time: f32, cfg: &Config) -> f32 {
    let t = (run_time / 720.0).min(1.0);
    let full_target = 0.72 + t * 0.18; // 0.72 → 0.90 over 12 minutes
    if cfg.spawn_ramp_duration <= 0.0 {
        return full_target;
    }
    let ramp = (run_time / cfg.spawn_ramp_duration).clamp(0.0, 1.0);
    cfg.spawn_ramp_start_coverage + (full_target - cfg.spawn_ramp_start_coverage) * ramp
}

#[allow(clippy::too_many_arguments)]
fn aabb_overlap(ax: f32, ay: f32, aw: f32, ah: f32, bx: f32, by: f32, bw: f32, bh: f32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}
