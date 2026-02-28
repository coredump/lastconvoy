use crate::orb::OrbType;
use serde::{Deserialize, Serialize};
use std::fs;

// ---------------------------------------------------------------------------
// Compile-time defaults
// ---------------------------------------------------------------------------

pub const SCREEN_W: u32 = 320;
pub const SCREEN_H: u32 = 180;
pub const WINDOW_SCALE: u32 = 3; // default window: 960×540
pub const MIN_SCALE: u32 = 1;
pub const MAX_SCALE: u32 = 6;

// Input
pub const ROTATE_INPUT: bool = false;
pub const ANALOG_DEADZONE: f32 = 0.2;
pub const TOUCH_STRIP_WIDTH_FRAC: f32 = 0.20;

// Lane row bounds (inclusive)
pub const TOP_BORDER_TOP: u32 = 0;
pub const TOP_BORDER_BOTTOM: u32 = 15;
pub const ENEMY_LANE_TOP: u32 = 16;
pub const ENEMY_LANE_BOTTOM: u32 = 119;
pub const DIVIDER_TOP: u32 = 120;
pub const DIVIDER_BOTTOM: u32 = 123;
pub const UPGRADE_LANE_TOP: u32 = 124;
pub const UPGRADE_LANE_BOTTOM: u32 = 163;
pub const BOTTOM_BORDER_TOP: u32 = 164;
pub const BOTTOM_BORDER_BOTTOM: u32 = 179;

// Projectile
pub const PROJECTILE_SPEED: f32 = 200.0;
pub const PROJECTILE_W: f32 = 4.0;
pub const PROJECTILE_H: f32 = 2.0;

// Player
pub const PLAYER_X: f32 = 8.0;
pub const PLAYER_SPEED: f32 = 90.0;
pub const PLAYER_FIRE_RATE: f32 = 0.18; // seconds between shots
pub const PLAYER_WIDTH: f32 = 24.0;
pub const PLAYER_HEIGHT: f32 = 16.0;
/// X coordinate where enemies stop (just in front of the player's right edge).
pub const BOUNDARY_X: f32 = PLAYER_X + PLAYER_WIDTH + 4.0;
pub const PLAYER_STARTING_SHIELDS: u32 = 0;

// Hit shake
pub const SHAKE_INTENSITY: f32 = 2.0; // max pixel offset
pub const SHAKE_DURATION: f32 = 0.15; // seconds

// Enemy sizes (w, h)
pub const ENEMY_SMALL_W: f32 = 16.0;
pub const ENEMY_SMALL_H: f32 = 16.0;
pub const ENEMY_MEDIUM_W: f32 = 24.0;
pub const ENEMY_MEDIUM_H: f32 = 24.0;
pub const ENEMY_HEAVY_W: f32 = 32.0;
pub const ENEMY_HEAVY_H: f32 = 24.0;
pub const ENEMY_LARGE_W: f32 = 40.0;
pub const ENEMY_LARGE_H: f32 = 32.0;
pub const ENEMY_ELITE_W: f32 = 48.0;
pub const ENEMY_ELITE_H: f32 = 40.0;

// Enemy HP
pub const ENEMY_SMALL_HP: i32 = 1;
pub const ENEMY_MEDIUM_HP: i32 = 4;
pub const ENEMY_HEAVY_HP: i32 = 7;
pub const ENEMY_LARGE_HP: i32 = 14;

// Enemy speeds (px/s, moving left)
pub const ENEMY_SMALL_SPEED: f32 = 40.0;
pub const ENEMY_MEDIUM_SPEED: f32 = 30.0;
pub const ENEMY_HEAVY_SPEED: f32 = 25.0;
pub const ENEMY_LARGE_SPEED: f32 = 20.0;

// Spawn intervals (seconds)
pub const ORB_SPAWN_INTERVAL: f32 = 5.0;
pub const MAX_ACTIVE_ORBS: usize = 1;

// Boundary
pub const BOUNDARY_SLOT_COUNT: usize = 3;
pub const BOUNDARY_DAMAGE_TICK: f32 = 2.0; // seconds between boundary damage ticks

// Orb
pub const ORB_ACTIVATION_HIT_COUNT: f32 = 5.0;
pub const ORB_ACTIVATION_DECAY_PER_SEC: f32 = 0.35;
pub const ORB_SPEED: f32 = 25.0;
pub const ORB_W: f32 = 20.0;
pub const ORB_H: f32 = 20.0;

// Elite / MiniBoss
pub const ELITE_INTERVAL: f32 = 60.0;
pub const ELITE_INTERVAL_JITTER: f32 = 10.0;
pub const MINIBOSS_INTERVAL: f32 = 180.0;
pub const MINIBOSS_INTERVAL_JITTER: f32 = 20.0;
pub const ELITE_HP: i32 = 20;
pub const MINIBOSS_HP: i32 = 30;
pub const ELITE_SPEED: f32 = 18.0;
pub const MINIBOSS_SPEED: f32 = 14.0;

// Coverage-based spawn system
pub const COVERAGE_ZONE_LEFT: f32 = 96.0;
pub const COVERAGE_ZONE_RIGHT: f32 = 320.0;
pub const COVERAGE_ZONE_WIDTH: f32 = 224.0; // COVERAGE_ZONE_RIGHT - COVERAGE_ZONE_LEFT
pub const COVERAGE_HYSTERESIS: f32 = 0.03;
pub const SPAWN_TICK_INTERVAL: f32 = 0.1; // 10 Hz
pub const SPAWN_SLOT_WIDTH: f32 = 20.0;
pub const SPAWN_SLOT_COUNT: usize = 11;
pub const SPAWN_LEAD_PX: f32 = 12.0;
pub const SPAWN_MAX_RETRIES: usize = 5;
pub const BIG_INJECT_BASE_INTERVAL: f32 = 2.2;

// Scaling rates (fractional increase per second)
pub const ENEMY_HP_SCALE: f32 = 0.001;
pub const SHIELDED_FREQ_SCALE: f32 = 0.001;
pub const SPEED_SCALE_PER_SEC: f32 = 0.0003; // +0.03%/s → 1.18× at 10min
pub const SPEED_SCALE_CAP: f32 = 1.5; // speed never exceeds 1.5× base
pub const HP_SCALE_HEAVY_MULT: f32 = 1.5; // heavy enemies scale HP 1.5× faster
pub const HP_SCALE_LARGE_MULT: f32 = 2.0; // large enemies scale HP 2× faster

// Medium/Large introduction times (seconds into run)
pub const MEDIUM_INTRO_TIME: f32 = 30.0;
pub const HEAVY_INTRO_TIME: f32 = 90.0;
pub const LARGE_INTRO_TIME: f32 = 150.0;

// Spawn ramp-up: ease into full density over the first few seconds
pub const SPAWN_RAMP_DURATION: f32 = 8.0; // seconds to reach full coverage target
pub const SPAWN_RAMP_START_COVERAGE: f32 = 0.15; // initial coverage target at t=0

// ---------------------------------------------------------------------------
// Debug
// ---------------------------------------------------------------------------

pub const DEBUG_LOG_GAMEPLAY: bool = false;
pub const DEBUG_LOG_FILE: &str = "";

pub const DAMAGE_LEVELS: [i32; 3] = [1, 2, 3];
pub const MAX_DAMAGE_LEVEL: usize = 3;
pub const DAMAGE_UPGRADE_APPLIES_TO_DRONES: bool = true;

pub const FIRE_RATE_LEVELS: [f32; 3] = [0.18, 0.14, 0.10];
pub const MAX_FIRE_RATE_LEVEL: usize = 3;
pub const FIRE_RATE_UPGRADE_APPLIES_TO_DRONES: bool = true;

pub const BURST_INTERVALS: [f32; 3] = [5.0, 3.5, 2.0];
pub const MAX_BURST_LEVEL: usize = 3;

pub const MAX_PIERCE_LEVEL: usize = 3;
pub const BURST_DAMAGE_MULTIPLIER: i32 = 2;

pub const MAX_STAGGER_LEVEL: usize = 1;
pub const STAGGER_KNOCKBACK_PX: f32 = 12.0;

// ---------------------------------------------------------------------------
// RuntimeConfig — all fields Optional; TOML file only needs overrides
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub player_speed: Option<f32>,
    pub player_fire_rate: Option<f32>,
    pub player_starting_shields: Option<u32>,

    pub enemy_small_speed: Option<f32>,
    pub enemy_medium_speed: Option<f32>,
    pub enemy_heavy_speed: Option<f32>,
    pub enemy_large_speed: Option<f32>,

    pub enemy_small_hp: Option<i32>,
    pub enemy_medium_hp: Option<i32>,
    pub enemy_heavy_hp: Option<i32>,
    pub enemy_large_hp: Option<i32>,

    pub orb_spawn_interval: Option<f32>,
    pub max_active_orbs: Option<usize>,

    pub boundary_slot_count: Option<usize>,
    pub boundary_damage_tick: Option<f32>,

    pub orb_speed: Option<f32>,

    pub elite_interval: Option<f32>,
    pub elite_interval_jitter: Option<f32>,
    pub miniboss_interval: Option<f32>,
    pub miniboss_interval_jitter: Option<f32>,
    pub elite_hp: Option<i32>,
    pub elite_speed: Option<f32>,
    pub miniboss_hp: Option<i32>,

    pub enemy_hp_scale: Option<f32>,
    pub shielded_freq_scale: Option<f32>,
    pub speed_scale_per_sec: Option<f32>,
    pub speed_scale_cap: Option<f32>,
    pub hp_scale_heavy_mult: Option<f32>,
    pub hp_scale_large_mult: Option<f32>,

    pub medium_intro_time: Option<f32>,
    pub heavy_intro_time: Option<f32>,
    pub large_intro_time: Option<f32>,

    pub spawn_ramp_duration: Option<f32>,
    pub spawn_ramp_start_coverage: Option<f32>,

    pub rotate_input: Option<bool>,
    pub analog_deadzone: Option<f32>,

    pub projectile_speed: Option<f32>,

    pub damage_levels: Option<Vec<i32>>,
    pub damage_upgrade_applies_to_drones: Option<bool>,

    pub fire_rate_levels: Option<Vec<f32>>,
    pub fire_rate_upgrade_applies_to_drones: Option<bool>,

    pub burst_intervals: Option<Vec<f32>>,
    pub burst_damage_multiplier: Option<i32>,

    // Debug flags
    pub debug_all_enemies: Option<bool>,
    pub debug_log_gameplay: Option<bool>,
    pub debug_log_file: Option<String>,
    pub debug_force_orb: Option<String>,
}

// ---------------------------------------------------------------------------
// Config — resolved (no Options)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Config {
    pub player_speed: f32,
    pub player_fire_rate: f32,
    pub player_starting_shields: u32,

    pub enemy_small_speed: f32,
    pub enemy_medium_speed: f32,
    pub enemy_heavy_speed: f32,
    pub enemy_large_speed: f32,

    pub enemy_small_hp: i32,
    pub enemy_medium_hp: i32,
    pub enemy_heavy_hp: i32,
    pub enemy_large_hp: i32,

    pub orb_spawn_interval: f32,
    pub max_active_orbs: usize,

    pub boundary_slot_count: usize,
    pub boundary_damage_tick: f32,

    pub orb_speed: f32,

    pub elite_interval: f32,
    pub elite_interval_jitter: f32,
    pub miniboss_interval: f32,
    pub miniboss_interval_jitter: f32,
    pub elite_hp: i32,
    pub elite_speed: f32,
    pub miniboss_hp: i32,

    pub enemy_hp_scale: f32,
    pub shielded_freq_scale: f32,
    pub speed_scale_per_sec: f32,
    pub speed_scale_cap: f32,
    pub hp_scale_heavy_mult: f32,
    pub hp_scale_large_mult: f32,

    pub medium_intro_time: f32,
    pub heavy_intro_time: f32,
    pub large_intro_time: f32,

    pub spawn_ramp_duration: f32,
    pub spawn_ramp_start_coverage: f32,

    pub rotate_input: bool,
    pub analog_deadzone: f32,

    pub projectile_speed: f32,

    pub damage_levels: [i32; 3],
    pub damage_upgrade_applies_to_drones: bool,

    pub fire_rate_levels: [f32; 3],
    pub fire_rate_upgrade_applies_to_drones: bool,

    pub burst_intervals: [f32; 3],
    pub burst_damage_multiplier: i32,

    /// Debug: spawn all enemy kinds from the start (bypasses intro timers).
    pub debug_all_enemies: bool,
    pub debug_log_gameplay: bool,
    pub debug_log_file: String,
    /// Debug: if Some, only this orb type spawns (bypasses level gates).
    pub debug_force_orb: Option<OrbType>,
}

impl Config {
    pub fn from_runtime(rt: RuntimeConfig) -> Self {
        Self {
            player_speed: rt.player_speed.unwrap_or(PLAYER_SPEED),
            player_fire_rate: rt.player_fire_rate.unwrap_or(PLAYER_FIRE_RATE),
            player_starting_shields: rt
                .player_starting_shields
                .unwrap_or(PLAYER_STARTING_SHIELDS),

            enemy_small_speed: rt.enemy_small_speed.unwrap_or(ENEMY_SMALL_SPEED),
            enemy_medium_speed: rt.enemy_medium_speed.unwrap_or(ENEMY_MEDIUM_SPEED),
            enemy_heavy_speed: rt.enemy_heavy_speed.unwrap_or(ENEMY_HEAVY_SPEED),
            enemy_large_speed: rt.enemy_large_speed.unwrap_or(ENEMY_LARGE_SPEED),

            enemy_small_hp: rt.enemy_small_hp.unwrap_or(ENEMY_SMALL_HP),
            enemy_medium_hp: rt.enemy_medium_hp.unwrap_or(ENEMY_MEDIUM_HP),
            enemy_heavy_hp: rt.enemy_heavy_hp.unwrap_or(ENEMY_HEAVY_HP),
            enemy_large_hp: rt.enemy_large_hp.unwrap_or(ENEMY_LARGE_HP),

            orb_spawn_interval: rt.orb_spawn_interval.unwrap_or(ORB_SPAWN_INTERVAL),
            max_active_orbs: rt.max_active_orbs.unwrap_or(MAX_ACTIVE_ORBS),

            boundary_slot_count: rt.boundary_slot_count.unwrap_or(BOUNDARY_SLOT_COUNT),
            boundary_damage_tick: rt.boundary_damage_tick.unwrap_or(BOUNDARY_DAMAGE_TICK),

            orb_speed: rt.orb_speed.unwrap_or(ORB_SPEED),

            elite_interval: rt.elite_interval.unwrap_or(ELITE_INTERVAL),
            elite_interval_jitter: rt.elite_interval_jitter.unwrap_or(ELITE_INTERVAL_JITTER),
            miniboss_interval: rt.miniboss_interval.unwrap_or(MINIBOSS_INTERVAL),
            miniboss_interval_jitter: rt
                .miniboss_interval_jitter
                .unwrap_or(MINIBOSS_INTERVAL_JITTER),
            elite_hp: rt.elite_hp.unwrap_or(ELITE_HP),
            elite_speed: rt.elite_speed.unwrap_or(ELITE_SPEED),
            miniboss_hp: rt.miniboss_hp.unwrap_or(MINIBOSS_HP),

            enemy_hp_scale: rt.enemy_hp_scale.unwrap_or(ENEMY_HP_SCALE),
            shielded_freq_scale: rt.shielded_freq_scale.unwrap_or(SHIELDED_FREQ_SCALE),
            speed_scale_per_sec: rt.speed_scale_per_sec.unwrap_or(SPEED_SCALE_PER_SEC),
            speed_scale_cap: rt.speed_scale_cap.unwrap_or(SPEED_SCALE_CAP),
            hp_scale_heavy_mult: rt.hp_scale_heavy_mult.unwrap_or(HP_SCALE_HEAVY_MULT),
            hp_scale_large_mult: rt.hp_scale_large_mult.unwrap_or(HP_SCALE_LARGE_MULT),

            medium_intro_time: rt.medium_intro_time.unwrap_or(MEDIUM_INTRO_TIME),
            heavy_intro_time: rt.heavy_intro_time.unwrap_or(HEAVY_INTRO_TIME),
            large_intro_time: rt.large_intro_time.unwrap_or(LARGE_INTRO_TIME),

            spawn_ramp_duration: rt.spawn_ramp_duration.unwrap_or(SPAWN_RAMP_DURATION),
            spawn_ramp_start_coverage: rt
                .spawn_ramp_start_coverage
                .unwrap_or(SPAWN_RAMP_START_COVERAGE),

            rotate_input: rt.rotate_input.unwrap_or(ROTATE_INPUT),
            analog_deadzone: rt.analog_deadzone.unwrap_or(ANALOG_DEADZONE),

            projectile_speed: rt.projectile_speed.unwrap_or(PROJECTILE_SPEED),

            damage_levels: {
                let v = rt.damage_levels.unwrap_or_default();
                if v.len() == 3 {
                    [v[0], v[1], v[2]]
                } else {
                    DAMAGE_LEVELS
                }
            },
            damage_upgrade_applies_to_drones: rt
                .damage_upgrade_applies_to_drones
                .unwrap_or(DAMAGE_UPGRADE_APPLIES_TO_DRONES),

            fire_rate_levels: {
                let v = rt.fire_rate_levels.unwrap_or_default();
                if v.len() == 3 {
                    [v[0], v[1], v[2]]
                } else {
                    FIRE_RATE_LEVELS
                }
            },
            fire_rate_upgrade_applies_to_drones: rt
                .fire_rate_upgrade_applies_to_drones
                .unwrap_or(FIRE_RATE_UPGRADE_APPLIES_TO_DRONES),

            burst_intervals: {
                let v = rt.burst_intervals.unwrap_or_default();
                if v.len() == 3 {
                    [v[0], v[1], v[2]]
                } else {
                    BURST_INTERVALS
                }
            },
            burst_damage_multiplier: rt
                .burst_damage_multiplier
                .unwrap_or(BURST_DAMAGE_MULTIPLIER),

            debug_all_enemies: rt.debug_all_enemies.unwrap_or(false),
            debug_log_gameplay: rt.debug_log_gameplay.unwrap_or(DEBUG_LOG_GAMEPLAY),
            debug_log_file: rt
                .debug_log_file
                .unwrap_or_else(|| DEBUG_LOG_FILE.to_string()),
            debug_force_orb: rt.debug_force_orb.as_deref().and_then(|s| {
                match s.to_lowercase().as_str() {
                    "burst" => Some(OrbType::Burst),
                    "damage" => Some(OrbType::Damage),
                    "defense" => Some(OrbType::Defense),
                    "drone" => Some(OrbType::Drone),
                    "firerate" => Some(OrbType::FireRate),
                    "pierce" => Some(OrbType::Pierce),
                    "stagger" => Some(OrbType::Stagger),
                    _ => None,
                }
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// load_runtime_config
// ---------------------------------------------------------------------------

pub fn load_runtime_config() -> RuntimeConfig {
    match fs::read_to_string("config.toml") {
        Ok(text) => match toml::from_str::<RuntimeConfig>(&text) {
            Ok(cfg) => cfg,
            Err(e) => panic!(
                "[config] Failed to parse config.toml: {e}\nFix the file or delete it to regenerate defaults."
            ),
        },
        Err(_) => RuntimeConfig::default(),
    }
}

// ---------------------------------------------------------------------------
// save_default_config_if_missing
// ---------------------------------------------------------------------------

pub fn save_default_config_if_missing() {
    if fs::metadata("config.toml").is_ok() {
        return;
    }
    let default = RuntimeConfig::default();
    match toml::to_string_pretty(&default) {
        Ok(text) => {
            if let Err(e) = fs::write("config.toml", text) {
                eprintln!("[config] Could not write config.toml: {e}");
            }
        }
        Err(e) => {
            eprintln!("[config] Could not serialize default config: {e}");
        }
    }
}
