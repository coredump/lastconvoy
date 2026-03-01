use macroquad::prelude::*;

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
    PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_X, PRE_BOUNDARY_STOP_OFFSET, PROJECTILE_H, PROJECTILE_W,
    SCREEN_W, SHIELDED_FREQ_SCALE, SPAWN_LEAD_PX, SPAWN_MAX_RETRIES, SPAWN_SLOT_COUNT,
    SPAWN_SLOT_WIDTH, SPAWN_TICK_INTERVAL, STAGGER_KNOCKBACK_PX, TOP_BORDER_BOTTOM, TOP_BORDER_TOP,
    UPGRADE_LANE_BOTTOM, UPGRADE_LANE_TOP,
};
use crate::drone::Drone;
use crate::elite::EliteEvent;
use crate::enemy::{Enemy, EnemyKind, EnemyState};
use crate::input::InputState;
use crate::orb::Orb;
use crate::orb::{OrbPhase, OrbType};
use crate::player::Player;
use crate::projectile::{Projectile, ProjectileSource};
use crate::shield::{ShieldHitResult, ShieldSystem};
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

pub struct BoundaryController {
    /// IDs of enemies currently in wind-up (Breaching state).
    pub breach_group: Vec<u64>,
    /// Game time when the first enemy in the current group entered Breaching.
    pub breach_start_time: f32,
    /// True when a breach is in progress; new arrivals must queue.
    pub breach_locked: bool,
    /// Countdown for explosive shield micro-stall (freezes enemy movement).
    pub stall_timer: f32,
}

impl BoundaryController {
    fn new() -> Self {
        Self {
            breach_group: Vec::new(),
            breach_start_time: 0.0,
            breach_locked: false,
            stall_timer: 0.0,
        }
    }

    fn reset(&mut self) {
        self.breach_group.clear();
        self.breach_start_time = 0.0;
        self.breach_locked = false;
        self.stall_timer = 0.0;
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
    pub orb_sprite_shield: Sprite,
    pub orb_sprite_drone: Sprite,
    pub orb_sprite_explosive: Sprite,
    pub orb_sprite_fire_rate: Sprite,
    pub orb_sprite_burst: Sprite,
    pub orb_sprite_pierce: Sprite,
    pub orb_sprite_stagger: Sprite,
    pub orb_sprite_seal: Sprite,
    pub damage_level: usize,
    pub fire_rate_level: usize,
    pub burst_level: usize,
    pub pierce_level: usize,
    pub stagger_level: usize,
    pub burst_timer: f32,
    pub burst_ready: bool,
    pub drones: Vec<Drone>,
    pub boundary_ctrl: BoundaryController,
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
        orb_sprite_shield: Sprite,
        orb_sprite_drone: Sprite,
        orb_sprite_explosive: Sprite,
        orb_sprite_fire_rate: Sprite,
        orb_sprite_burst: Sprite,
        orb_sprite_pierce: Sprite,
        orb_sprite_stagger: Sprite,
        orb_sprite_seal: Sprite,
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
            orb_sprite_shield,
            orb_sprite_drone,
            orb_sprite_explosive,
            orb_sprite_fire_rate,
            orb_sprite_burst,
            orb_sprite_pierce,
            orb_sprite_stagger,
            orb_sprite_seal,
            damage_level: 0,
            fire_rate_level: 0,
            burst_level: 0,
            pierce_level: 0,
            stagger_level: 0,
            burst_timer: 0.0,
            burst_ready: false,
            drones: Vec::new(),
            boundary_ctrl: BoundaryController::new(),
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
        self.boundary_ctrl.reset();
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
        match self.shields.take_hit() {
            ShieldHitResult::NoShield => {
                self.game_over = true;
            }
            ShieldHitResult::ExplosiveBreak => {
                self.trigger_explosive_shield();
            }
            ShieldHitResult::NormalAbsorbed => {}
        }
        let remaining = self.shields.count();
        self.dlog(&format!(
            "PLAYER_DMG count=1 shields_remaining={}",
            remaining
        ));
    }

    /// Explosive shield segment broke: kill enemies in the clear zone, push Large/Elite back.
    fn trigger_explosive_shield(&mut self) {
        let clear_distance = self.config.explosive_shield_clear_distance;
        let zone_right = BOUNDARY_X + clear_distance;
        let lane_top = ENEMY_LANE_TOP as f32;
        let lane_bottom = ENEMY_LANE_BOTTOM as f32;

        let mut stagger_targets: Vec<usize> = Vec::new();
        for (i, e) in self.enemies.iter_mut().enumerate() {
            // Enemy overlaps explosion zone horizontally and vertically.
            if e.x < zone_right
                && e.x + e.width > BOUNDARY_X
                && e.y + e.height > lane_top
                && e.y < lane_bottom
            {
                match e.kind {
                    EnemyKind::Large | EnemyKind::Elite => {
                        stagger_targets.push(i);
                    }
                    _ => {
                        // Kill immediately; slot release handled in the dead-enemy pass.
                        e.hp = 0;
                    }
                }
            }
        }

        // Push Large/Elite enemies back using the same stagger logic.
        for idx in stagger_targets {
            if self.enemies[idx].stagger_immune {
                continue;
            }
            let id = self.enemies[idx].id;
            self.enemies[idx].stagger_immune = true;
            self.enemies[idx].x += STAGGER_KNOCKBACK_PX;
            self.enemies[idx].state = EnemyState::Moving;
            self.boundary_ctrl.breach_group.retain(|&bid| bid != id);
            if self.enemies[idx].x + self.enemies[idx].width > SCREEN_W as f32 {
                self.enemies[idx].x = SCREEN_W as f32 - self.enemies[idx].width;
            }
        }
        // Clear breach lock if group emptied by explosive.
        if self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
        }
        // Apply micro-stall.
        self.boundary_ctrl.stall_timer = self.config.explosive_micro_stall;

        self.dlog(&format!(
            "EXPLOSIVE_SHIELD zone=[{:.0}..{:.0}] lane=[{}..{}]",
            BOUNDARY_X, zone_right, ENEMY_LANE_TOP, ENEMY_LANE_BOTTOM
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
        // Indices of enemies that should receive stagger knockback this frame.
        let mut stagger_targets: Vec<usize> = Vec::new();
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
            for (ei, e) in self.enemies.iter_mut().enumerate() {
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
                        && !e.stagger_immune
                        && self.stagger_level > 0
                        && matches!(
                            e.kind,
                            EnemyKind::Small | EnemyKind::Medium | EnemyKind::Heavy
                        )
                        && !(e.kind == EnemyKind::Small && e.hp <= 3 * player_dmg)
                    {
                        stagger_targets.push(ei);
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

        // Apply deferred stagger: always push full STAGGER_KNOCKBACK_PX, then forward-cascade
        // any enemies that now overlap the displaced enemy (chain push).
        // Marks stagger_immune so the effect only applies once per enemy.
        for idx in &stagger_targets {
            let idx = *idx;
            if self.enemies[idx].stagger_immune {
                continue;
            }
            let id = self.enemies[idx].id;
            self.enemies[idx].stagger_immune = true;
            self.enemies[idx].x += STAGGER_KNOCKBACK_PX;
            self.enemies[idx].state = EnemyState::Moving;
            self.boundary_ctrl.breach_group.retain(|&bid| bid != id);
            // Clamp to screen right edge.
            if self.enemies[idx].x + self.enemies[idx].width > SCREEN_W as f32 {
                self.enemies[idx].x = SCREEN_W as f32 - self.enemies[idx].width;
            }
        }
        // Unlock boundary if stagger emptied the breach group.
        if self.boundary_ctrl.breach_locked && self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
        }
        // Forward cascade pass: resolve overlaps created by the stagger pushes.
        // Sort enemies by x ascending so each left enemy can push the one to its right.
        if !stagger_targets.is_empty() {
            let n = self.enemies.len();
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|&a, &b| self.enemies[a].x.partial_cmp(&self.enemies[b].x).unwrap());
            for ii in 0..order.len() {
                let left_idx = order[ii];
                let left_right = self.enemies[left_idx].x + self.enemies[left_idx].width;
                let left_y = self.enemies[left_idx].y;
                let left_h = self.enemies[left_idx].height;
                for &right_idx in &order[ii + 1..] {
                    let (re_y, re_h, re_x) = {
                        let re = &self.enemies[right_idx];
                        (re.y, re.height, re.x)
                    };
                    // Only push if they vertically overlap.
                    if re_y >= left_y + left_h || re_y + re_h <= left_y {
                        continue;
                    }
                    if re_x < left_right {
                        self.enemies[right_idx].x = left_right;
                        // Clamp to screen right edge.
                        if self.enemies[right_idx].x + self.enemies[right_idx].width
                            > SCREEN_W as f32
                        {
                            self.enemies[right_idx].x =
                                SCREEN_W as f32 - self.enemies[right_idx].width;
                        }
                    }
                }
            }
        }

        self.projectiles.retain(|p| !p.should_remove());

        // Remove dead enemies from breach group.
        self.boundary_ctrl
            .breach_group
            .retain(|id| self.enemies.iter().any(|e| e.id == *id && !e.is_dead()));
        if self.boundary_ctrl.breach_group.is_empty() && self.boundary_ctrl.breach_locked {
            // Last breacher died (e.g. killed by player mid-windup): unlock the boundary.
            self.boundary_ctrl.breach_locked = false;
        }

        self.enemies.retain(|e| !e.is_dead() || e.shake.is_active());

        // Coverage-based enemy spawning
        self.spawn_ctrl.tick_accum += dt;
        while self.spawn_ctrl.tick_accum >= SPAWN_TICK_INTERVAL {
            self.spawn_ctrl.tick_accum -= SPAWN_TICK_INTERVAL;
            self.tick_spawn();
        }

        // Tick micro-stall and skip enemy movement while stalling.
        if self.boundary_ctrl.stall_timer > 0.0 {
            self.boundary_ctrl.stall_timer = (self.boundary_ctrl.stall_timer - dt).max(0.0);
        }
        let stalling = self.boundary_ctrl.stall_timer > 0.0;

        // Update enemies (movement gated by state and stall).
        for e in &mut self.enemies {
            if stalling {
                // Freeze all movement during micro-stall; still update shake.
                e.shake.update(dt);
            } else {
                e.update(dt);
            }
        }

        // Boundary arrival: transition Moving enemies that reached BOUNDARY_X.
        let simultaneous_window = self.config.simultaneous_breach_window;
        let now = self.run_time;
        let mut arrivals: Vec<usize> = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.state == EnemyState::Moving && e.x <= BOUNDARY_X {
                arrivals.push(i);
            }
        }
        for i in arrivals {
            let in_window = self.boundary_ctrl.breach_locked
                && (now - self.boundary_ctrl.breach_start_time) <= simultaneous_window;
            if !self.boundary_ctrl.breach_locked || in_window {
                let id = self.enemies[i].id;
                self.enemies[i].state = EnemyState::Breaching;
                self.enemies[i].x = BOUNDARY_X;
                self.enemies[i].windup_elapsed = 0.0;
                self.boundary_ctrl.breach_group.push(id);
                if !self.boundary_ctrl.breach_locked {
                    self.boundary_ctrl.breach_locked = true;
                    self.boundary_ctrl.breach_start_time = now;
                }
                self.dlog(&format!(
                    "BREACH_START {:?} id={} windup={:.2}s",
                    self.enemies[i].kind, id, self.enemies[i].windup_time
                ));
            } else {
                // Breach locked: clamp enemy out of the boundary zone; it stays Moving
                // and compresses naturally behind the breaching enemy via stacking.
                self.enemies[i].x = (BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET).max(self.enemies[i].x);
            }
        }

        // Tick wind-up for breaching enemies; collect resolved breaches.
        let mut resolved_ids: Vec<u64> = Vec::new();
        for e in &mut self.enemies {
            if e.state == EnemyState::Breaching {
                e.windup_elapsed += dt;
                if e.windup_elapsed >= e.windup_time {
                    resolved_ids.push(e.id);
                }
            }
        }

        // Resolve each completed breach: deal damage, despawn enemy.
        for id in resolved_ids {
            if let Some(pos) = self.enemies.iter().position(|e| e.id == id) {
                let kind = self.enemies[pos].kind;
                self.enemies[pos].hp = 0;
                self.boundary_ctrl.breach_group.retain(|&bid| bid != id);
                self.dlog(&format!("BREACH_RESOLVE {:?} id={}", kind, id));
                self.take_player_damage();
                if self.game_over {
                    return;
                }
            }
        }

        // Unlock boundary once breach group is fully resolved.
        // No explicit queue release needed: enemies remain Moving and compress naturally
        // behind the breacher; the frontmost will reach BOUNDARY_X and start the next breach.
        if self.boundary_ctrl.breach_locked && self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
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

        // Boundary clamp pass: after stacking, prevent Moving enemies from drifting into the
        // locked boundary zone (stacking could push a follower forward past the stop line).
        if self.boundary_ctrl.breach_locked {
            let stop_x = BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET;
            for e in &mut self.enemies {
                if e.state == EnemyState::Moving && e.x < stop_x {
                    e.x = stop_x;
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
                // Build weighted pool; gate Shield if shields are full.
                let shields_full = self.shields.count() >= crate::shield::MAX_SHIELD_SEGMENTS;
                let mut pool: Vec<OrbType> = Vec::with_capacity(5);
                if self.burst_level < MAX_BURST_LEVEL {
                    pool.push(OrbType::Burst);
                }
                if self.damage_level < MAX_DAMAGE_LEVEL {
                    pool.push(OrbType::Damage);
                }
                if !shields_full {
                    pool.push(OrbType::Shield);
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
                if !self.shields.has_explosive() {
                    pool.push(OrbType::Explosive);
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

        // Projectile-orb collision (player and drone shots).
        // Shots are consumed on contact with any orb regardless of phase.
        // Inactive orbs also receive a hit that advances their activation progress.
        for p in &mut self.projectiles {
            if !p.alive {
                continue;
            }
            for o in &mut self.orbs {
                if aabb_overlap(
                    p.x,
                    p.y,
                    PROJECTILE_W,
                    PROJECTILE_H,
                    o.x,
                    o.y,
                    o.width,
                    o.height,
                ) {
                    if o.phase == OrbPhase::Inactive {
                        o.hit_this_frame = true;
                    }
                    p.alive = false;
                    break;
                }
            }
        }

        // Orb movement
        for o in &mut self.orbs {
            o.update(dt);
        }

        // Player-orb collection (active orbs only).
        // INVARIANT: at most one orb of each type may be collected per frame.
        // Spawn spacing and orb speed make simultaneous same-type collection impossible in
        // normal gameplay. Logic below is NOT designed to handle multiples correctly —
        // explosive in particular calls convert_to_explosive() only once regardless of count.
        let mut shield_grants = 0u32;
        let mut damage_collected = 0u32;
        let mut fire_rate_collected = 0u32;
        let mut burst_collected = 0u32;
        let mut pierce_collected = 0u32;
        let mut stagger_collected = 0u32;
        let mut explosive_collected = 0u32;
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
                    OrbType::Shield => {
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
                    OrbType::Explosive => {
                        explosive_collected += 1;
                    }
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
        if explosive_collected > 0 {
            self.shields.convert_to_explosive();
            self.dlog("ORB_COLLECT Explosive");
        }

        let player_cx = self.player.x + self.player.width / 2.0;
        self.orbs.retain(|o| {
            let orb_cx = o.x + o.width / 2.0;
            orb_cx >= player_cx && !o.is_collected()
        });

        // Advance sprite animations
        self.orb_sprite_damage.update(dt);
        self.orb_sprite_shield.update(dt);
        self.orb_sprite_drone.update(dt);
        self.orb_sprite_explosive.update(dt);
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
            let shield_tint = if self.shields.has_explosive() {
                Color::from_rgba(255, 140, 0, 255) // orange tint for explosive
            } else {
                WHITE
            };
            self.boundary_shield_sprite.draw_tinted(
                BOUNDARY_X + self.shields.shake.offset_x(),
                ENEMY_LANE_TOP as f32,
                shield_tint,
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
            let tint = if e.state == EnemyState::Breaching {
                e.windup_tint()
            } else {
                e.flash.tint()
            };
            sprite.draw_tinted(e.x + e.shake.offset_x(), e.y, tint);
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
        // Tag indices in upgrades.json: 1=damage, 2=rate, 3=burst, 4=pierce,
        //   5=stagger, 6=extradrone, 8=shield, 9=explosive  (see main.rs)
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
                OrbType::Explosive => &mut self.orb_sprite_explosive,
                OrbType::FireRate => &mut self.orb_sprite_fire_rate,
                OrbType::Pierce => &mut self.orb_sprite_pierce,
                OrbType::Stagger => &mut self.orb_sprite_stagger,
            };
            if inactive {
                // Draw orb at full brightness, then dim overlay, then seal on top —
                // so the seal blends against the dim layer rather than the grey orb.
                sprite.draw_tinted_frozen(sx, sy, WHITE);
                draw_rectangle(sx, sy, 20.0, 20.0, Color::from_rgba(0, 0, 0, 130));
                // Draw seal overlay based on activation progress.
                if activation_progress < 1.0 {
                    let frame = (activation_progress * 4.0).floor().clamp(0.0, 3.0) as u32;
                    // Draw the stable inner segments (frame+1) always, then blink
                    // only the outermost segment (frame) on top.
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
        // Simple overlay — dim the screen and prompt restart.
        let overlay = Color::from_rgba(0, 0, 0, 160);
        draw_rectangle(0.0, 0.0, SCREEN_W as f32, 180.0, overlay);
        // No text rendering in Phase 1 (no font loaded) — the blank screen + shield HUD
        // at 0 segments is sufficient feedback. Restart on Space/Enter/R.
    }

    /// Draw shield segment count in the top-left corner as small green squares.
    fn draw_shield_hud(&self) {
        let size = 4.0_f32;
        let gap = 2.0_f32;
        let start_x = 2.0_f32;
        let y = 2.0_f32;
        for (i, seg) in self.shields.segments.iter().enumerate() {
            let x = start_x + i as f32 * (size + gap);
            let color = if seg.explosive {
                Color::from_rgba(255, 140, 0, 255) // orange for explosive
            } else {
                Color::from_rgba(0, 200, 80, 255) // green for normal
            };
            draw_rectangle(x, y, size, size, color);
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
            let kind = if let Some(forced) = self.config.debug_force_enemy {
                forced
            } else if self.config.debug_all_enemies || self.run_time >= LARGE_INTRO_TIME {
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
            let kind = self.config.debug_force_enemy.unwrap_or(EnemyKind::Small);
            self.try_place_enemy(kind);
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

        let windup_time = match kind {
            EnemyKind::Small => self.config.windup_time_small,
            EnemyKind::Medium => self.config.windup_time_medium,
            EnemyKind::Heavy => self.config.windup_time_heavy,
            EnemyKind::Large => self.config.windup_time_large,
            EnemyKind::Elite => self.config.windup_time_elite,
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
            let mut enemy = Enemy::new(x, y, kind, hp, speed, w, h, windup_time);
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
        // Boundary marker: subtle vertical line across the enemy lane
        draw_rectangle(
            BOUNDARY_X,
            ENEMY_LANE_TOP as f32,
            1.0,
            (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32,
            steel_dark,
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
