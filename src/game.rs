use macroquad::prelude::*;

use crate::config::SHAKE_DURATION;
use crate::config::SHAKE_INTENSITY;
use crate::config::{
    BASE_DAMAGE_VALUE, BIG_INJECT_BASE_INTERVAL, BOTTOM_BORDER_BOTTOM, BOTTOM_BORDER_TOP,
    BOUNDARY_X, COVERAGE_HYSTERESIS, COVERAGE_ZONE_LEFT, COVERAGE_ZONE_RIGHT, COVERAGE_ZONE_WIDTH,
    Config, DRONE_FIRE_RATE, DRONE_HEIGHT, DRONE_REMOTE_HEIGHT, DRONE_REMOTE_WIDTH,
    DRONE_Y_OFFSETS, ENEMY_ELITE_H, ENEMY_ELITE_W, ENEMY_HEAVY_H, ENEMY_HEAVY_HP,
    ENEMY_HEAVY_SPEED, ENEMY_HEAVY_W, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, ENEMY_LARGE_H,
    ENEMY_LARGE_HP, ENEMY_LARGE_SPEED, ENEMY_LARGE_W, ENEMY_MEDIUM_H, ENEMY_MEDIUM_HP,
    ENEMY_MEDIUM_SPEED, ENEMY_MEDIUM_W, ENEMY_SMALL_H, ENEMY_SMALL_HP, ENEMY_SMALL_SPEED,
    ENEMY_SMALL_W, HEAVY_INTRO_TIME, LARGE_INTRO_TIME, MAX_ATTACHED_DRONES, MEDIUM_INTRO_TIME,
    ORB_H, ORB_W, PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_X, PRE_BOUNDARY_STOP_OFFSET, PROJECTILE_H,
    PROJECTILE_SPEED, PROJECTILE_W, SCREEN_W, SHIELDED_FREQ_SCALE, SHOT_BARRIER_BOTTOM_Y,
    SHOT_BARRIER_GATE_X_MAX, SHOT_BARRIER_TOP_Y, SPAWN_LEAD_PX, SPAWN_MAX_RETRIES,
    SPAWN_SLOT_COUNT, SPAWN_SLOT_WIDTH, SPAWN_TICK_INTERVAL, STAGGER_KNOCKBACK_PX,
    TOP_BORDER_BOTTOM, TOP_BORDER_TOP, TOP_UPGRADE_LANE_BOTTOM, TOP_UPGRADE_LANE_TOP,
    UPGRADE_LANE_BOTTOM, UPGRADE_LANE_TOP,
};
use crate::drone::{Drone, RemoteDrone, RemoteDroneLane};
use crate::elite::EliteEvent;
use crate::enemy::{Enemy, EnemyKind, EnemyState};
use crate::input::InputState;
use crate::orb::Orb;
use crate::orb::{OrbPhase, OrbType};
use crate::player::Player;
use crate::projectile::{Projectile, ProjectileSource};
use crate::shield::{ShieldHitResult, ShieldSystem};
use crate::sprite::Sprite;
use crate::text::BitmapFont;

const FLOATING_TEXT_TTL: f32 = 0.8;
const FLOATING_TEXT_VY: f32 = -18.0;
const HUD_TIMER_BAR_W: f32 = 12.0;
const HUD_TIMER_BAR_H: f32 = 2.0;

pub struct FloatingText {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub ttl: f32,
    pub life: f32,
    pub color: Color,
}

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
    /// Cooldown after a natural breach resolution; blocks new breaches until expired.
    pub rebreach_cooldown: f32,
}

impl BoundaryController {
    fn new() -> Self {
        Self {
            breach_group: Vec::new(),
            breach_start_time: 0.0,
            breach_locked: false,
            stall_timer: 0.0,
            rebreach_cooldown: 0.0,
        }
    }

    fn reset(&mut self) {
        self.breach_group.clear();
        self.breach_start_time = 0.0;
        self.breach_locked = false;
        self.stall_timer = 0.0;
        self.rebreach_cooldown = 0.0;
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
    pub rail_wall_sprite: Sprite,
    pub bg_texture: Texture2D,
    pub bg_scroll_offsets: [f32; 3],
    pub shot_sprite: Sprite,
    pub burst_shot_sprite: Sprite,
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
    pub orb_sprite_drone_remote: Sprite,
    pub drone_sprite: Sprite,
    pub drone_remote_sprite: Sprite,
    pub damage_buff_t: f32,
    pub fire_rate_buff_t: f32,
    pub burst_buff_t: f32,
    pub pierce_buff_t: f32,
    pub stagger_buff_t: f32,
    pub burst_timer: f32,
    pub burst_ready: bool,
    pub drones: Vec<Drone>,
    pub remote_drones: Vec<RemoteDrone>,
    pub boundary_ctrl: BoundaryController,
    pub elite_event: EliteEvent,
    pub input: InputState,
    pub spawn_ctrl: SpawnController,
    pub orb_spawn_timer: f32,
    pub elite_timer: f32,
    pub miniboss_timer: f32,
    pub run_id: u32,
    pub run_time: f32,
    pub game_over: bool,
    pub kills_total: u32,
    pub breaches_total: u32,
    pub balance_log_timer: f32,
    pub debug_log: Option<crate::debug_log::DebugLog>,
    pub additive_material: Material,
    pub ui_font: BitmapFont,
    pub floating_texts: Vec<FloatingText>,
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
        shot_sprite: Sprite,
        burst_shot_sprite: Sprite,
        orb_sprite_damage: Sprite,
        orb_sprite_shield: Sprite,
        orb_sprite_drone: Sprite,
        orb_sprite_explosive: Sprite,
        orb_sprite_fire_rate: Sprite,
        orb_sprite_burst: Sprite,
        orb_sprite_pierce: Sprite,
        orb_sprite_stagger: Sprite,
        orb_sprite_seal: Sprite,
        orb_sprite_drone_remote: Sprite,
        drone_sprite: Sprite,
        drone_remote_sprite: Sprite,
        rail_wall_sprite: Sprite,
        bg_texture: Texture2D,
        ui_font: BitmapFont,
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

        let mut state = Self {
            player,
            player_sprite,
            enemy_small_sprite,
            enemy_medium_sprite,
            enemy_heavy_sprite,
            enemy_large_sprite,
            enemy_elite_sprite,
            boundary_shield_sprite,
            shot_sprite,
            burst_shot_sprite,
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
            orb_sprite_drone_remote,
            drone_sprite,
            drone_remote_sprite,
            rail_wall_sprite,
            bg_texture,
            bg_scroll_offsets: [0.0; 3],
            damage_buff_t: 0.0,
            fire_rate_buff_t: 0.0,
            burst_buff_t: 0.0,
            pierce_buff_t: 0.0,
            stagger_buff_t: 0.0,
            burst_timer: 0.0,
            burst_ready: false,
            drones: Vec::new(),
            remote_drones: Vec::new(),
            boundary_ctrl: BoundaryController::new(),
            elite_event: EliteEvent::new(),
            input: InputState::new(),
            spawn_ctrl: SpawnController::new(),
            orb_spawn_timer: 0.0,
            elite_timer: config.elite_interval,
            miniboss_timer: config.miniboss_interval,
            run_id: 1,
            run_time: 0.0,
            game_over: false,
            kills_total: 0,
            breaches_total: 0,
            balance_log_timer: 0.0,
            debug_log: if config.debug_log_gameplay {
                Some(crate::debug_log::DebugLog::new(
                    &config.debug_log_file.clone(),
                ))
            } else {
                None
            },
            config,
            ui_font,
            floating_texts: Vec::new(),
            additive_material: {
                use miniquad::{BlendFactor, BlendState, BlendValue, Equation};
                load_material(
                    ShaderSource::Glsl {
                        vertex: r#"#version 100
                            attribute vec3 position;
                            attribute vec2 texcoord;
                            attribute vec4 color0;
                            attribute vec4 normal;
                            varying lowp vec2 uv;
                            varying lowp vec4 color;
                            uniform mat4 Model;
                            uniform mat4 Projection;
                            void main() {
                                gl_Position = Projection * Model * vec4(position, 1);
                                color = color0 / 255.0;
                                uv = texcoord;
                            }"#,
                        fragment: r#"#version 100
                            varying lowp vec4 color;
                            varying lowp vec2 uv;
                            uniform sampler2D Texture;
                            void main() {
                                gl_FragColor = color * texture2D(Texture, uv);
                            }"#,
                    },
                    MaterialParams {
                        pipeline_params: PipelineParams {
                            color_blend: Some(BlendState::new(
                                Equation::Add,
                                BlendFactor::Value(BlendValue::SourceAlpha),
                                BlendFactor::One,
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                )
                .expect("additive material")
            },
        };
        state.log_run_start("boot");
        state
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
        self.remote_drones.clear();
        self.damage_buff_t = 0.0;
        self.fire_rate_buff_t = 0.0;
        self.player.fire_rate = self.config.player_fire_rate;
        self.burst_buff_t = 0.0;
        self.pierce_buff_t = 0.0;
        self.stagger_buff_t = 0.0;
        self.burst_timer = 0.0;
        self.burst_ready = false;
        self.elite_event = EliteEvent::new();
        self.spawn_ctrl.reset();
        self.orb_spawn_timer = 0.0;
        self.elite_timer = self.config.elite_interval;
        self.miniboss_timer = self.config.miniboss_interval;
        self.run_id = self.run_id.saturating_add(1);
        self.run_time = 0.0;
        self.game_over = false;
        self.kills_total = 0;
        self.breaches_total = 0;
        self.balance_log_timer = 0.0;
        self.floating_texts.clear();
        self.log_run_start("restart");
    }

    fn dlog(&mut self, msg: &str) {
        if let Some(log) = &mut self.debug_log {
            log.log(self.run_time, msg);
        }
    }

    fn log_run_start(&mut self, source: &str) {
        self.dlog(&format!(
            "RUN_START run_id={} source={}",
            self.run_id, source
        ));
    }

    fn log_run_end(&mut self, reason: &str) {
        self.dlog(&format!(
            "RUN_END run_id={} reason={} time_s={:.1} kills={} breaches={} shields={}",
            self.run_id,
            reason,
            self.run_time,
            self.kills_total,
            self.breaches_total,
            self.shields.count()
        ));
    }

    fn damage_buff_active(&self) -> bool {
        self.damage_buff_t > 0.0
    }

    fn fire_rate_buff_active(&self) -> bool {
        self.fire_rate_buff_t > 0.0
    }

    fn burst_buff_active(&self) -> bool {
        self.burst_buff_t > 0.0
    }

    fn pierce_buff_active(&self) -> bool {
        self.pierce_buff_t > 0.0
    }

    fn stagger_buff_active(&self) -> bool {
        self.stagger_buff_t > 0.0 && self.config.buff_stagger_enabled
    }

    fn current_damage(&self) -> f32 {
        if self.damage_buff_active() {
            self.config.buff_damage_value
        } else {
            BASE_DAMAGE_VALUE
        }
    }

    fn current_fire_rate(&self) -> f32 {
        if self.fire_rate_buff_active() {
            self.config.buff_fire_rate_value
        } else {
            self.config.player_fire_rate
        }
    }

    fn current_pierce(&self) -> i32 {
        if self.pierce_buff_active() {
            self.config.buff_pierce_value
        } else {
            0
        }
    }

    fn tick_buff_timers(&mut self, dt: f32) {
        self.damage_buff_t = (self.damage_buff_t - dt).max(0.0);
        self.fire_rate_buff_t = (self.fire_rate_buff_t - dt).max(0.0);
        self.burst_buff_t = (self.burst_buff_t - dt).max(0.0);
        self.pierce_buff_t = (self.pierce_buff_t - dt).max(0.0);
        self.stagger_buff_t = (self.stagger_buff_t - dt).max(0.0);
    }

    /// Compute estimated sustained DPS at current buffs (no burst, no pierce).
    fn dps_estimate(&self) -> f32 {
        let dmg = self.current_damage();
        let fire_rate = self.current_fire_rate();
        dmg / fire_rate
    }

    /// Compute time-to-kill a Large enemy at current HP scaling, in seconds.
    fn large_ttk(&self) -> f32 {
        let base_hp = self.config.enemy_large_hp as f32;
        let hp_mult =
            1.0 + self.config.enemy_hp_scale * self.run_time * self.config.hp_scale_large_mult;
        let large_hp = (base_hp * hp_mult).round().max(1.0);
        let dmg = self.current_damage();
        let fire_rate = self.current_fire_rate();
        large_hp * fire_rate / dmg
    }

    fn log_balance_snapshot(&mut self) {
        let dps = self.dps_estimate();
        let ttk = self.large_ttk();
        let base_hp = self.config.enemy_large_hp as f32;
        let hp_mult =
            1.0 + self.config.enemy_hp_scale * self.run_time * self.config.hp_scale_large_mult;
        let large_hp = (base_hp * hp_mult).round().max(1.0) as i32;
        let med_hp_mult = 1.0 + self.config.enemy_hp_scale * self.run_time;
        let medium_hp = (self.config.enemy_medium_hp as f32 * med_hp_mult)
            .round()
            .max(1.0) as i32;
        let buffs_active = self.damage_buff_active() as u8
            + self.fire_rate_buff_active() as u8
            + self.burst_buff_active() as u8
            + self.pierce_buff_active() as u8
            + self.stagger_buff_active() as u8;
        let pressure_bpm = if self.run_time > 0.0 {
            self.breaches_total as f32 * 60.0 / self.run_time
        } else {
            0.0
        };
        self.dlog(&format!(
            "BALANCE_SNAPSHOT dps={:.2} ttk_large_s={:.2} hp_large={} hp_medium={} \
             kills={} breaches={} pressure_bpm={:.2} shields={} buffs_active={} \
             buff_damage_s={:.1} buff_rate_s={:.1} buff_burst_s={:.1} buff_pierce_s={:.1} buff_stagger_s={:.1}",
            dps,
            ttk,
            large_hp,
            medium_hp,
            self.kills_total,
            self.breaches_total,
            pressure_bpm,
            self.shields.count(),
            buffs_active,
            self.damage_buff_t,
            self.fire_rate_buff_t,
            self.burst_buff_t,
            self.pierce_buff_t,
            self.stagger_buff_t,
        ));
    }

    /// Deal one damage event to the player: consume one shield segment, or die if none remain.
    fn take_player_damage(&mut self) {
        self.player.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
        match self.shields.take_hit() {
            ShieldHitResult::NoShield => {
                if !self.game_over {
                    self.log_run_end("death");
                }
                self.game_over = true;
            }
            ShieldHitResult::ExplosiveBreak => {
                self.trigger_explosive_shield();
            }
            ShieldHitResult::NormalAbsorbed => {}
        }
        let remaining = self.shields.count();
        self.dlog(&format!("PLAYER_HIT shields_after={}", remaining));
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
            "EXPLOSIVE_TRIGGER zone_x0={:.0} zone_x1={:.0} lane_y0={} lane_y1={}",
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
        self.update_floating_texts(dt);
        self.tick_buff_timers(dt);
        self.player.fire_rate = self.current_fire_rate();

        // Balance snapshot every 30s
        self.balance_log_timer -= dt;
        if self.balance_log_timer <= 0.0 {
            self.balance_log_timer = 30.0;
            self.log_balance_snapshot();
        }

        self.input.update(&self.config);

        // Player movement
        let axis = self.input.axis;
        let has_top_drone = !self.drones.is_empty();
        let has_bottom_drone = self.drones.len() >= 2;
        self.player
            .update(axis, dt, has_top_drone, has_bottom_drone);

        // Burst timer
        if self.burst_buff_active() {
            self.burst_timer -= dt;
            if self.burst_timer <= 0.0 {
                self.burst_ready = true;
                self.burst_timer = self.config.buff_burst_interval;
            }
        } else {
            self.burst_ready = false;
            self.burst_timer = 0.0;
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
                self.current_pierce(),
            ));
        }

        // Update projectiles
        for p in &mut self.projectiles {
            p.update(dt);
        }
        for p in &mut self.projectiles {
            if p.alive && Self::projectile_hits_shot_barrier(p) {
                p.alive = false;
            }
        }

        // Projectile-enemy collision (player + drone + remote drone shots).
        let mut kill_logs: Vec<String> = Vec::new();
        // Indices of enemies that should receive stagger knockback this frame.
        let mut stagger_targets: Vec<usize> = Vec::new();
        let base_dmg = self.current_damage();
        let stagger_enabled = self.stagger_buff_active();
        for p in &mut self.projectiles {
            if !p.alive {
                continue;
            }
            let proj_dmg_base = if matches!(
                p.source,
                ProjectileSource::Drone | ProjectileSource::RemoteDrone
            ) && !self.config.damage_upgrade_applies_to_drones
            {
                BASE_DAMAGE_VALUE
            } else {
                base_dmg
            };
            let proj_dmg = (if p.is_burst {
                proj_dmg_base * self.config.burst_damage_multiplier
            } else {
                proj_dmg_base
            })
            .round() as i32;
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
                    e.take_damage(proj_dmg);
                    if !e.is_dead()
                        && !e.stagger_immune
                        && stagger_enabled
                        && matches!(
                            e.kind,
                            EnemyKind::Small | EnemyKind::Medium | EnemyKind::Heavy
                        )
                        && !(e.kind == EnemyKind::Small && e.hp <= 3 * proj_dmg)
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
        self.kills_total += kill_logs.len() as u32;
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
        if self.boundary_ctrl.rebreach_cooldown > 0.0 {
            self.boundary_ctrl.rebreach_cooldown =
                (self.boundary_ctrl.rebreach_cooldown - dt).max(0.0);
        }

        // Update enemies (movement gated by state and stall).
        for e in &mut self.enemies {
            if stalling {
                // Freeze all movement during micro-stall; still update shake.
                e.shake.update(dt);
            } else {
                let prev_x = e.x;
                e.update(dt);
                // While breach is locked, stop Moving enemies at the stop line.
                // Enemies already past it keep their pre-move position (no push-back).
                if self.boundary_ctrl.breach_locked && e.state == EnemyState::Moving {
                    let stop_x = BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET;
                    e.x = e.x.max(stop_x.min(prev_x));
                }
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
            let cooldown_expired = self.boundary_ctrl.rebreach_cooldown <= 0.0;
            if (!self.boundary_ctrl.breach_locked && cooldown_expired) || in_window {
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
                    "BREACH_START kind={:?} id={} windup_s={:.2}",
                    self.enemies[i].kind, id, self.enemies[i].windup_time
                ));
            } else {
                // Breach locked or cooldown active: clamp enemy at appropriate stop line.
                let stop_x = if self.boundary_ctrl.breach_locked {
                    BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET
                } else {
                    BOUNDARY_X
                };
                self.enemies[i].x = stop_x.max(self.enemies[i].x);
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
        let had_resolution = !resolved_ids.is_empty();
        for id in resolved_ids {
            if let Some(pos) = self.enemies.iter().position(|e| e.id == id) {
                let kind = self.enemies[pos].kind;
                self.enemies[pos].hp = 0;
                self.boundary_ctrl.breach_group.retain(|&bid| bid != id);
                self.breaches_total += 1;
                self.dlog(&format!("BREACH_RESOLVE kind={:?} id={}", kind, id));
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
            if had_resolution {
                self.boundary_ctrl.rebreach_cooldown = self.config.re_breach_cooldown;
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

        // During re-breach cooldown, hold Moving enemies at the boundary line.
        if self.boundary_ctrl.rebreach_cooldown > 0.0 {
            for e in &mut self.enemies {
                if e.state == EnemyState::Moving && e.x < BOUNDARY_X {
                    e.x = BOUNDARY_X;
                }
            }
        }

        // Orb spawning
        self.orb_spawn_timer -= dt;
        if self.orb_spawn_timer <= 0.0 {
            self.orb_spawn_timer = self.config.orb_spawn_interval;
            if self.orbs.len() < self.config.max_active_orbs {
                // Build weighted pool; gate Shield if shields are full.
                let shields_full = self.shields.count() >= crate::shield::MAX_SHIELD_SEGMENTS;
                // Temporary offense buffs are excluded while active.
                let mut pool: Vec<(OrbType, u32)> = Vec::with_capacity(8);
                if !self.burst_buff_active() {
                    pool.push((OrbType::Burst, 1));
                }
                if !self.damage_buff_active() {
                    pool.push((OrbType::Damage, 1));
                }
                if !shields_full {
                    pool.push((OrbType::Shield, 1));
                }
                let drone_remaining =
                    (MAX_ATTACHED_DRONES.saturating_sub(self.drones.len())) as u32;
                if drone_remaining > 0 {
                    pool.push((OrbType::Drone, drone_remaining));
                }
                pool.push((OrbType::DroneRemote, 1));
                if !self.fire_rate_buff_active() {
                    pool.push((OrbType::FireRate, 1));
                }
                if !self.pierce_buff_active() {
                    pool.push((OrbType::Pierce, 1));
                }
                if !self.stagger_buff_active() {
                    pool.push((OrbType::Stagger, 1));
                }
                if !self.shields.has_explosive() {
                    pool.push((OrbType::Explosive, 1));
                }
                if let Some(forced) = self.config.debug_force_orb {
                    pool = vec![(forced, 1)];
                }
                if !pool.is_empty() {
                    let total: u32 = pool.iter().map(|(_, w)| w).sum();
                    let roll_orb_type = || -> OrbType {
                        let mut roll = rand::gen_range(0u32, total);
                        pool.iter()
                            .find(|(_, w)| {
                                if roll < *w {
                                    true
                                } else {
                                    roll -= w;
                                    false
                                }
                            })
                            .map(|(t, _)| *t)
                            .unwrap_or(pool[0].0)
                    };
                    let remaining = self.config.max_active_orbs.saturating_sub(self.orbs.len());
                    if remaining >= 2 {
                        let top_type = roll_orb_type();
                        let bottom_type = roll_orb_type();
                        self.orbs.push(Orb::new(
                            SCREEN_W as f32,
                            self.upgrade_lane_mid_top() - ORB_H / 2.0,
                            ORB_W,
                            ORB_H,
                            self.config.orb_speed,
                            top_type,
                        ));
                        self.orbs.push(Orb::new(
                            SCREEN_W as f32,
                            self.upgrade_lane_mid_bottom() - ORB_H / 2.0,
                            ORB_W,
                            ORB_H,
                            self.config.orb_speed,
                            bottom_type,
                        ));
                    } else if remaining == 1 {
                        let top_mid = self.upgrade_lane_mid_top();
                        let bottom_mid = self.upgrade_lane_mid_bottom();
                        let top_count = self
                            .orbs
                            .iter()
                            .filter(|orb| orb.y + ORB_H / 2.0 < ENEMY_LANE_TOP as f32)
                            .count();
                        let bottom_count = self
                            .orbs
                            .iter()
                            .filter(|orb| orb.y + ORB_H / 2.0 > ENEMY_LANE_BOTTOM as f32)
                            .count();
                        let lane_mid = if top_count < bottom_count {
                            top_mid
                        } else if bottom_count < top_count {
                            bottom_mid
                        } else if rand::gen_range(0u32, 2) == 0 {
                            top_mid
                        } else {
                            bottom_mid
                        };
                        self.orbs.push(Orb::new(
                            SCREEN_W as f32,
                            lane_mid - ORB_H / 2.0,
                            ORB_W,
                            ORB_H,
                            self.config.orb_speed,
                            roll_orb_type(),
                        ));
                    }
                }
            }
        }

        // Projectile-orb collision — all shots interact with orbs equally.
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

        // Orb movement + activation detection (used by remote drone despawn).
        let mut orb_activated_this_frame = false;
        for o in &mut self.orbs {
            let was_inactive = o.phase == OrbPhase::Inactive;
            o.update(dt);
            if was_inactive && o.phase == OrbPhase::Active {
                orb_activated_this_frame = true;
            }
        }

        // Player-orb collection (active orbs only).
        // Track pickup origin positions for floating upgrade text.
        let mut pickups: Vec<(OrbType, f32, f32)> = Vec::new();
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
                pickups.push((o.orb_type, o.x + o.width * 0.5, o.y));
            }
        }

        for (orb_type, px, py) in pickups {
            let popup_tag = match orb_type {
                OrbType::Damage => {
                    self.damage_buff_t = self.config.buff_damage_duration;
                    self.dlog(&format!(
                        "ORB_PICKUP type=DamageBuff ttl_s={:.1}",
                        self.damage_buff_t
                    ));
                    Some("+DMG")
                }
                OrbType::FireRate => {
                    self.fire_rate_buff_t = self.config.buff_fire_rate_duration;
                    self.player.fire_rate = self.current_fire_rate();
                    self.dlog(&format!(
                        "ORB_PICKUP type=FireRateBuff ttl_s={:.1}",
                        self.fire_rate_buff_t
                    ));
                    Some("+RATE")
                }
                OrbType::Burst => {
                    let was_active = self.burst_buff_active();
                    self.burst_buff_t = self.config.buff_burst_duration;
                    if !was_active {
                        self.burst_timer = self.config.buff_burst_interval;
                        self.burst_ready = false;
                    }
                    self.dlog(&format!(
                        "ORB_PICKUP type=BurstBuff ttl_s={:.1}",
                        self.burst_buff_t
                    ));
                    Some("+BURST")
                }
                OrbType::Pierce => {
                    self.pierce_buff_t = self.config.buff_pierce_duration;
                    self.dlog(&format!(
                        "ORB_PICKUP type=PierceBuff ttl_s={:.1}",
                        self.pierce_buff_t
                    ));
                    Some("+PIERCE")
                }
                OrbType::Stagger => {
                    self.stagger_buff_t = self.config.buff_stagger_duration;
                    self.dlog(&format!(
                        "ORB_PICKUP type=StaggerBuff ttl_s={:.1}",
                        self.stagger_buff_t
                    ));
                    Some("+STAGGER")
                }
                OrbType::Shield => {
                    let before = self.shields.count();
                    self.shields.add_segments(1);
                    let after = self.shields.count();
                    if after > before {
                        Some("+SHIELD")
                    } else {
                        None
                    }
                }
                OrbType::Explosive => {
                    let before_has = self.shields.has_explosive();
                    let before_count = self.shields.count();
                    self.shields.convert_to_explosive();
                    let after_has = self.shields.has_explosive();
                    let after_count = self.shields.count();
                    if after_has && !before_has {
                        self.dlog("ORB_PICKUP type=Explosive");
                        Some("+EXPLOSIVE")
                    } else if after_count > before_count {
                        self.dlog("ORB_PICKUP type=ShieldFromExplosive");
                        Some("+SHIELD")
                    } else {
                        None
                    }
                }
                OrbType::Drone => {
                    if self.drones.len() < MAX_ATTACHED_DRONES {
                        let index = self.drones.len();
                        let dy = DRONE_Y_OFFSETS[index.min(DRONE_Y_OFFSETS.len() - 1)];
                        self.drones
                            .push(Drone::new(self.player.x, self.player.y + dy));
                        self.dlog(&format!("ORB_PICKUP type=Drone index={}", index));
                        Some("+DRONE")
                    } else {
                        None
                    }
                }
                OrbType::DroneRemote => {
                    let top_y = self.upgrade_lane_mid_top() - DRONE_REMOTE_HEIGHT / 2.0;
                    let bottom_y = self.upgrade_lane_mid_bottom() - DRONE_REMOTE_HEIGHT / 2.0;
                    self.remote_drones.push(RemoteDrone::new(
                        BOUNDARY_X,
                        top_y,
                        RemoteDroneLane::Top,
                    ));
                    self.remote_drones.push(RemoteDrone::new(
                        BOUNDARY_X,
                        bottom_y,
                        RemoteDroneLane::Bottom,
                    ));
                    self.dlog("ORB_PICKUP type=DroneRemote");
                    Some("+REMOTE")
                }
            };

            if let Some(tag) = popup_tag {
                self.spawn_upgrade_floating_text(tag, px, py);
            }
        }

        self.orbs
            .retain(|o| o.x + o.width > 0.0 && !o.is_collected());

        // Update attached drones: follow player, fire into enemy lane.
        let drone_fire_rate = if self.config.fire_rate_upgrade_applies_to_drones {
            self.current_fire_rate()
        } else {
            DRONE_FIRE_RATE
        };
        let drone_pierce = if self.config.damage_upgrade_applies_to_drones {
            self.current_pierce()
        } else {
            0
        };
        let mut drone_shots: Vec<Projectile> = Vec::new();
        for (i, drone) in self.drones.iter_mut().enumerate() {
            let dy = DRONE_Y_OFFSETS[i.min(DRONE_Y_OFFSETS.len() - 1)];
            drone.x = self.player.x;
            drone.y = self.player.y + dy;
            drone.fire_timer -= dt;
            if drone.fire_timer <= 0.0 {
                drone.fire_timer = drone_fire_rate;
                let proj_x = self.player.x + PLAYER_WIDTH;
                let proj_y = drone.y + DRONE_HEIGHT / 2.0 - PROJECTILE_H / 2.0;
                drone_shots.push(Projectile::new(
                    proj_x,
                    proj_y,
                    self.config.projectile_speed,
                    ProjectileSource::Drone,
                    false,
                    drone_pierce,
                ));
            }
        }
        self.projectiles.extend(drone_shots);

        // Update remote drones: stationary, fire rightward continuously.
        // Despawn immediately when any orb activates this frame.
        if orb_activated_this_frame {
            self.remote_drones.clear();
        } else {
            let mut rd_shots: Vec<Projectile> = Vec::new();
            for rd in &mut self.remote_drones {
                rd.fire_timer -= dt;
                if rd.fire_timer <= 0.0 {
                    rd.fire_timer = DRONE_FIRE_RATE;
                    rd_shots.push(Projectile::new(
                        rd.x + DRONE_REMOTE_WIDTH,
                        rd.y + DRONE_REMOTE_HEIGHT / 2.0 - PROJECTILE_H / 2.0,
                        PROJECTILE_SPEED,
                        ProjectileSource::RemoteDrone,
                        false,
                        0,
                    ));
                }
            }
            self.projectiles.extend(rd_shots);
        }

        // Advance parallax scroll offsets
        let bg_speeds = [
            self.config.bg_parallax_speed_back,
            self.config.bg_parallax_speed_stars,
            self.config.bg_parallax_speed_props,
        ];
        for (off, &spd) in self.bg_scroll_offsets.iter_mut().zip(bg_speeds.iter()) {
            *off += spd * dt;
        }

        // Advance sprite animations
        self.orb_sprite_damage.update(dt);
        self.orb_sprite_shield.update(dt);
        self.orb_sprite_drone.update(dt);
        self.orb_sprite_drone_remote.update(dt);
        self.orb_sprite_explosive.update(dt);
        self.orb_sprite_fire_rate.update(dt);
        self.orb_sprite_burst.update(dt);
        self.orb_sprite_pierce.update(dt);
        self.orb_sprite_stagger.update(dt);
        self.drone_sprite.update(dt);
        self.drone_remote_sprite.update(dt);
        self.shot_sprite.update(dt);
        self.burst_shot_sprite.update(dt);
        self.player_sprite.update(dt);
        self.enemy_small_sprite.update(dt);
        self.enemy_medium_sprite.update(dt);
        self.enemy_heavy_sprite.update(dt);
        self.enemy_large_sprite.update(dt);
        self.enemy_elite_sprite.update(dt);
        self.boundary_shield_sprite.update(dt);
        self.rail_wall_sprite.update(dt);
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
            let shield_y = TOP_UPGRADE_LANE_TOP as f32; // 21.0
            let shield_h = UPGRADE_LANE_BOTTOM as f32 - shield_y + 1.0; // 138.0
            self.boundary_shield_sprite.draw_3slice_vertical(
                BOUNDARY_X - 3.0 + self.shields.shake.offset_x(),
                shield_y,
                shield_h,
                "top",
                "mid",
                "bot",
                shield_tint,
            );
        }
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
            let sprite = if p.is_burst {
                &self.burst_shot_sprite
            } else {
                &self.shot_sprite
            };
            sprite.draw(p.x, p.y - 0.5);
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
                WHITE
            };
            let draw_x = e.x + e.shake.offset_x();
            sprite.draw_tinted(draw_x, e.y, tint);
            let flash_color = e.flash.tint();
            if flash_color != WHITE {
                sprite.draw_additive(draw_x, e.y, flash_color, 0.7, &self.additive_material);
            }
        }

        self.draw_orbs();

        self.draw_shield_hud();
        self.draw_upgrade_hud();
        self.draw_run_timer_hud();
        self.draw_floating_texts();

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
                OrbType::DroneRemote => &mut self.orb_sprite_drone_remote,
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
        let title = "GAME OVER";
        let time_line = format!("TIME {}", self.format_run_timer());
        let kb_line = format!(
            "KILLS {}  BREACHES {}",
            self.kills_total, self.breaches_total
        );
        let restart = "PRESS SPACE/ENTER/R";
        let restart2 = "TO RESTART";

        let title_size = self.ui_font.measure(title, 2, 1);
        let title_x = (SCREEN_W as f32 - title_size.x) * 0.5;
        self.ui_font.draw(
            title,
            title_x,
            60.0,
            2,
            Color::from_rgba(255, 90, 90, 255),
            1,
        );

        let time_size = self.ui_font.measure(&time_line, 2, 1);
        let time_x = (SCREEN_W as f32 - time_size.x) * 0.5;
        self.ui_font.draw(
            &time_line,
            time_x,
            88.0,
            2,
            Color::from_rgba(245, 245, 245, 255),
            1,
        );

        let kb_size = self.ui_font.measure(&kb_line, 1, 1);
        let kb_x = (SCREEN_W as f32 - kb_size.x) * 0.5;
        self.ui_font.draw(
            &kb_line,
            kb_x,
            116.0,
            1,
            Color::from_rgba(220, 220, 220, 255),
            1,
        );

        let restart_size = self.ui_font.measure(restart, 1, 1);
        let restart_x = (SCREEN_W as f32 - restart_size.x) * 0.5;
        self.ui_font.draw(
            restart,
            restart_x,
            134.0,
            1,
            Color::from_rgba(240, 240, 180, 255),
            1,
        );
        let restart2_size = self.ui_font.measure(restart2, 1, 1);
        let restart2_x = (SCREEN_W as f32 - restart2_size.x) * 0.5;
        self.ui_font.draw(
            restart2,
            restart2_x,
            150.0,
            1,
            Color::from_rgba(240, 240, 180, 255),
            1,
        );
    }

    /// Draw shield segment count in the top-left corner as small squares.
    /// Dark backdrop squares are drawn first for all MAX_SHIELD_SEGMENTS slots.
    fn draw_shield_hud(&self) {
        let size = 4.0_f32;
        let gap = 2.0_f32;
        let start_x = 2.0_f32;
        let y = 2.0_f32;
        let dark = Color::from_rgba(40, 40, 40, 255);
        for i in 0..crate::shield::MAX_SHIELD_SEGMENTS {
            let x = start_x + i as f32 * (size + gap);
            draw_rectangle(x, y, size, size, dark);
        }
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

    /// Draw collected upgrade icons in the HUD after the shield area.
    fn draw_upgrade_hud(&mut self) {
        let shield_area_end = 2.0 + crate::shield::MAX_SHIELD_SEGMENTS as f32 * (4.0 + 2.0) + 2.0;
        let icon_size = 12.0_f32;
        let icon_gap = 2.0_f32;
        let y = 2.0_f32;
        let mut x = shield_area_end;
        {
            let ratio = if self.config.buff_damage_duration > 0.0 {
                (self.damage_buff_t / self.config.buff_damage_duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if self.damage_buff_active() {
                self.orb_sprite_damage
                    .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(40, 40, 40, 255),
                );
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W * ratio,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(150, 255, 190, 255),
                );
                x += icon_size + icon_gap;
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
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(40, 40, 40, 255),
                );
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W * ratio,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(150, 255, 190, 255),
                );
                x += icon_size + icon_gap;
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
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(40, 40, 40, 255),
                );
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W * ratio,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(150, 255, 190, 255),
                );
                x += icon_size + icon_gap;
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
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(40, 40, 40, 255),
                );
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W * ratio,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(150, 255, 190, 255),
                );
                x += icon_size + icon_gap;
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
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(40, 40, 40, 255),
                );
                draw_rectangle(
                    x,
                    y + icon_size + 1.0,
                    HUD_TIMER_BAR_W * ratio,
                    HUD_TIMER_BAR_H,
                    Color::from_rgba(150, 255, 190, 255),
                );
                x += icon_size + icon_gap;
            }
        }

        if self.shields.count() > self.config.player_starting_shields as usize {
            self.orb_sprite_shield
                .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
            x += icon_size + icon_gap;
        }
        if self.shields.segments.iter().any(|s| s.explosive) {
            self.orb_sprite_explosive
                .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
            x += icon_size + icon_gap;
        }
        if !self.drones.is_empty() {
            self.orb_sprite_drone
                .draw_frozen_scaled(x, y, icon_size, icon_size, WHITE);
        }
    }

    fn format_run_timer(&self) -> String {
        let total = self.run_time.max(0.0).floor() as u32;
        let minutes = total / 60;
        let seconds = total % 60;
        format!("{minutes:02}:{seconds:02}")
    }

    fn upgrade_lane_mid_top(&self) -> f32 {
        (TOP_UPGRADE_LANE_TOP + TOP_UPGRADE_LANE_BOTTOM + 1) as f32 / 2.0
    }

    fn upgrade_lane_mid_bottom(&self) -> f32 {
        (UPGRADE_LANE_TOP + UPGRADE_LANE_BOTTOM + 1) as f32 / 2.0
    }

    fn projectile_hits_shot_barrier(p: &Projectile) -> bool {
        if p.x <= SHOT_BARRIER_GATE_X_MAX {
            return false;
        }
        let y0 = p.y;
        let y1 = p.y + PROJECTILE_H;
        let overlaps_top = y0 < SHOT_BARRIER_TOP_Y + 1.0 && y1 > SHOT_BARRIER_TOP_Y;
        let overlaps_bottom = y0 < SHOT_BARRIER_BOTTOM_Y + 1.0 && y1 > SHOT_BARRIER_BOTTOM_Y;
        overlaps_top || overlaps_bottom
    }

    fn draw_run_timer_hud(&self) {
        let timer = self.format_run_timer();
        let size = self.ui_font.measure(&timer, 1, 1);
        let x = SCREEN_W as f32 - 2.0 - size.x;
        self.ui_font
            .draw(&timer, x, 2.0, 1, Color::from_rgba(220, 220, 220, 255), 1);
    }

    fn spawn_upgrade_floating_text(&mut self, tag: &str, x: f32, y: f32) {
        let width = self.ui_font.measure(tag, 1, 1).x;
        self.floating_texts.push(FloatingText {
            text: tag.to_string(),
            x: x - width * 0.5,
            y,
            vy: FLOATING_TEXT_VY,
            ttl: FLOATING_TEXT_TTL,
            life: FLOATING_TEXT_TTL,
            color: Color::from_rgba(230, 255, 180, 255),
        });
    }

    fn update_floating_texts(&mut self, dt: f32) {
        for t in &mut self.floating_texts {
            t.y += t.vy * dt;
            t.life = (t.life - dt).max(0.0);
        }
        self.floating_texts.retain(|t| t.life > 0.0);
    }

    fn draw_floating_texts(&self) {
        for t in &self.floating_texts {
            let alpha = (t.life / t.ttl).clamp(0.0, 1.0);
            let mut color = t.color;
            color.a *= alpha;
            self.ui_font.draw(&t.text, t.x, t.y, 1, color, 1);
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

        // Keep injecting medium/heavy/large threats even when coverage is near target.
        // Without this slack, mid/late runs can plateau into mostly-small traffic.
        let inject_coverage_cap = (target + 0.10).min(1.0);
        if self.spawn_ctrl.inject_timer <= 0.0 && coverage < inject_coverage_cap {
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
            // Ramp big-inject cadence over time so late runs keep must-react enemies.
            let late_pressure = (self.run_time / 600.0).clamp(0.0, 1.0); // full effect by 10 min
            let interval_scale = 1.0 - 0.25 * late_pressure; // up to 25% faster
            // Reset timer with small jitter
            let jitter = rand::gen_range(-0.3_f32, 0.3);
            self.spawn_ctrl.inject_timer = BIG_INJECT_BASE_INTERVAL * interval_scale + jitter;
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
            self.enemies.push(enemy);
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

        // Top border (rows 0–20): Steel Dark fill + Steel Mid accent on inner edge
        let tb_y = TOP_BORDER_TOP as f32;
        let tb_h = (TOP_BORDER_BOTTOM - TOP_BORDER_TOP + 1) as f32;
        draw_rectangle(0.0, tb_y, w, tb_h, steel_dark);
        draw_rectangle(0.0, tb_y + tb_h - 1.0, w, 1.0, steel_mid);

        // Top upgrade lane (rows 21–42): Upgrade Teal Very Dark
        draw_rectangle(
            0.0,
            TOP_UPGRADE_LANE_TOP as f32,
            w,
            (TOP_UPGRADE_LANE_BOTTOM - TOP_UPGRADE_LANE_TOP + 1) as f32,
            teal_very_dark,
        );

        // Enemy lane (rows 43–136): Space Very Dark
        draw_rectangle(
            0.0,
            ENEMY_LANE_TOP as f32,
            w,
            (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32,
            space_very_dark,
        );
        // Bottom upgrade lane (rows 137–158): Upgrade Teal Very Dark
        draw_rectangle(
            0.0,
            UPGRADE_LANE_TOP as f32,
            w,
            (UPGRADE_LANE_BOTTOM - UPGRADE_LANE_TOP + 1) as f32,
            teal_very_dark,
        );

        // Bottom border (rows 159–179): Steel Dark fill + Steel Mid accent on inner edge
        let bb_y = BOTTOM_BORDER_TOP as f32;
        let bb_h = (BOTTOM_BORDER_BOTTOM - BOTTOM_BORDER_TOP + 1) as f32;
        draw_rectangle(0.0, bb_y, w, bb_h, steel_dark);
        draw_rectangle(0.0, bb_y, w, 1.0, steel_mid);

        // Parallax background layers (blue.png: 284×480, 3 layers of 160px stacked vertically).
        // Layer 0 (back): static — drawn once at BOUNDARY_X, no scroll.
        // Layers 1 (stars) and 2 (props): tiled horizontally with wrapping scroll.
        {
            let lane_top = ENEMY_LANE_TOP as f32;
            let lane_h = (ENEMY_LANE_BOTTOM - ENEMY_LANE_TOP + 1) as f32; // 94.0
            let layer_h = 160.0_f32;
            let clip_y = (layer_h - lane_h) / 2.0; // 33.0
            let tex_w = 284.0_f32;
            for (i, &offset) in self.bg_scroll_offsets.iter().enumerate() {
                let src_y = i as f32 * layer_h + clip_y;
                let src = DrawTextureParams {
                    source: Some(Rect::new(0.0, src_y, tex_w, lane_h)),
                    dest_size: Some(vec2(tex_w, lane_h)),
                    ..Default::default()
                };
                if i == 0 {
                    // Back layer: single static copy
                    draw_texture_ex(&self.bg_texture, BOUNDARY_X, lane_top, WHITE, src);
                } else {
                    // Stars / props: tiled with seamless wrap; skip if speed is 0 (disabled)
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
                        &self.bg_texture,
                        BOUNDARY_X + wrapped - tex_w,
                        lane_top,
                        WHITE,
                        src.clone(),
                    );
                    draw_texture_ex(
                        &self.bg_texture,
                        BOUNDARY_X + wrapped,
                        lane_top,
                        WHITE,
                        src,
                    );
                }
            }
        }

        // Rail wall: tile the animated sprite (36×36) from x=0; right edge aligns with BOUNDARY_X.
        // Only draw within the non-border area (y=21–158); clip the last partial tile.
        let rail_x = 0.0_f32; // left-aligned to screen edge
        let tile_h = 36.0_f32;
        let rail_start = TOP_UPGRADE_LANE_TOP as f32; // 21.0
        let rail_end = UPGRADE_LANE_BOTTOM as f32 + 1.0; // 159.0 (exclusive)
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
