// Compile-time defaults and runtime config loading from TOML.
// serde, toml, crate types
use crate::enemy::EnemyKind;
use crate::orb::OrbType;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Compile-time defaults
// ---------------------------------------------------------------------------

pub const SCREEN_W: u32 = 320;
pub const SCREEN_H: u32 = 180;
// Default window size is 960×540.
pub const WINDOW_SCALE: u32 = 3;
pub const MIN_SCALE: u32 = 1;
pub const MAX_SCALE: u32 = 6;

// Input
pub const ROTATE_INPUT: bool = false;
pub const ANALOG_DEADZONE: f32 = 0.2;
pub const TOUCH_STRIP_WIDTH_FRAC: f32 = 0.20;

// Lane row bounds (inclusive)
pub const TOP_BORDER_TOP: u32 = 0;
pub const TOP_BORDER_BOTTOM: u32 = 20;
pub const TOP_UPGRADE_LANE_TOP: u32 = 21;
pub const TOP_UPGRADE_LANE_BOTTOM: u32 = 42;
pub const ENEMY_LANE_TOP: u32 = 43;
pub const ENEMY_LANE_BOTTOM: u32 = 157;
pub const UPGRADE_LANE_TOP: u32 = 158;
pub const UPGRADE_LANE_BOTTOM: u32 = 179;

// Projectile
pub const PROJECTILE_SPEED: f32 = 200.0;
pub const PROJECTILE_W: f32 = 7.0;
pub const PROJECTILE_H: f32 = 3.0;
/// Shots crossing lane boundaries are blocked except within this left-side corridor.
pub const SHOT_BARRIER_GATE_X_MAX: f32 = 32.0;
/// 1px shot barrier on the upgrade-lane side of the top enemy/upgrade boundary.
pub const SHOT_BARRIER_TOP_Y: f32 = TOP_UPGRADE_LANE_BOTTOM as f32;
/// 1px shot barrier on the upgrade-lane side of the bottom enemy/upgrade boundary.
pub const SHOT_BARRIER_BOTTOM_Y: f32 = UPGRADE_LANE_TOP as f32;

// Player
pub const PLAYER_X: f32 = 8.0;
pub const PLAYER_SPEED: f32 = 90.0;
pub const PLAYER_FIRE_RATE: f32 = 0.18;
pub const PLAYER_WIDTH: f32 = 24.0;
pub const PLAYER_HEIGHT: f32 = 16.0;
/// X coordinate where enemies stop (just in front of the player's right edge).
pub const BOUNDARY_X: f32 = PLAYER_X + PLAYER_WIDTH + 4.0;
pub const PLAYER_STARTING_SHIELDS: u32 = 0;

// Hit shake
pub const SHAKE_INTENSITY: f32 = 2.0;
pub const SHAKE_DURATION: f32 = 0.15;

// Enemy flash colors and timing
pub const DAMAGE_FLASH_COLOR: macroquad::prelude::Color =
    macroquad::prelude::Color::new(0.7, 1.0, 1.0, 1.0);
pub const DAMAGE_FLASH_DURATION: f32 = 0.017;
pub const DAMAGE_FLASH_COOLDOWN: f32 = 0.08;
pub const WINDUP_FLASH_COLOR: macroquad::prelude::Color =
    macroquad::prelude::Color::new(1.0, 0.85, 0.85, 1.0);
pub const WINDUP_FLASH_FREQ_MIN: f32 = 2.0;
pub const WINDUP_FLASH_FREQ_MAX: f32 = 10.0;

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
pub const ORB_SPAWN_INTERVAL: f32 = 4.0;
pub const MAX_ACTIVE_ORBS: usize = 2;

// Boundary / breach
/// Wind-up duration (seconds) before each enemy kind resolves its breach event.
pub const WINDUP_TIME_SMALL: f32 = 0.8;
pub const WINDUP_TIME_MEDIUM: f32 = 0.5;
pub const WINDUP_TIME_HEAVY: f32 = 1.0;
pub const WINDUP_TIME_LARGE: f32 = 1.3;
pub const WINDUP_TIME_ELITE: f32 = 1.0;
/// If a second enemy arrives at the boundary within this window of the first, it joins the breach group.
pub const SIMULTANEOUS_BREACH_WINDOW: f32 = 0.10;
/// Micro-stall duration applied to enemy movement after an explosive shield detonates.
pub const EXPLOSIVE_MICRO_STALL: f32 = 0.25;
/// How far right of BOUNDARY_X enemies are clamped while breach is locked (visible pressure cluster).
pub const PRE_BOUNDARY_STOP_OFFSET: f32 = 24.0;
/// Cooldown after a breach resolves naturally before a new breach can begin.
pub const RE_BREACH_COOLDOWN: f32 = 0.4;

// Orb
pub const ORB_ACTIVATION_HIT_COUNT: f32 = 5.0;
pub const ORB_ACTIVATION_DECAY_PER_SEC: f32 = 0.35;
pub const ORB_SPEED: f32 = 25.0;
pub const ORB_W: f32 = 20.0;
pub const ORB_H: f32 = 20.0;
pub const SEAL_BLINK_DELAY: f32 = 0.3;
pub const SEAL_BLINK_RATE: f32 = 0.12;

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
pub const COVERAGE_ZONE_WIDTH: f32 = 224.0;
pub const COVERAGE_HYSTERESIS: f32 = 0.03;
// Spawn tick rate: 10 Hz.
pub const SPAWN_TICK_INTERVAL: f32 = 0.1;
pub const SPAWN_SLOT_WIDTH: f32 = 20.0;
pub const SPAWN_SLOT_COUNT: usize = 11;
pub const SPAWN_LEAD_PX: f32 = 12.0;
pub const SPAWN_MAX_RETRIES: usize = 5;
pub const BIG_INJECT_BASE_INTERVAL: f32 = 2.2;

// Scaling rates (fractional increase per second)
pub const ENEMY_HP_SCALE: f32 = 0.003;
pub const SHIELDED_FREQ_SCALE: f32 = 0.001;
// +0.03%/s → 1.18× speed multiplier at 10 min.
pub const SPEED_SCALE_PER_SEC: f32 = 0.0003;
pub const SPEED_SCALE_CAP: f32 = 1.5;
// Heavy enemies scale HP 2× faster than small.
pub const HP_SCALE_HEAVY_MULT: f32 = 2.0;
// Large enemies scale HP 3× faster than small.
pub const HP_SCALE_LARGE_MULT: f32 = 3.0;

// Medium/Large introduction times (seconds into run)
pub const MEDIUM_INTRO_TIME: f32 = 30.0;
pub const HEAVY_INTRO_TIME: f32 = 90.0;
pub const LARGE_INTRO_TIME: f32 = 150.0;

// Spawn ramp-up: ease into full density over the first few seconds
// Seconds to ease from SPAWN_RAMP_START_COVERAGE to full density.
pub const SPAWN_RAMP_DURATION: f32 = 8.0;
// Coverage target at run start (t=0), before ramp completes.
pub const SPAWN_RAMP_START_COVERAGE: f32 = 0.15;

// ---------------------------------------------------------------------------
// Debug
// ---------------------------------------------------------------------------

pub const DEBUG_LOG_GAMEPLAY: bool = false;
pub const DEBUG_LOG_FILE: &str = "";

pub const BUFF_DAMAGE_DURATION: f32 = 14.0;
pub const BUFF_FIRE_RATE_DURATION: f32 = 12.0;
pub const BUFF_BURST_DURATION: f32 = 16.0;
pub const BUFF_PIERCE_DURATION: f32 = 10.0;
pub const BUFF_STAGGER_DURATION: f32 = 12.0;

pub const BUFF_DAMAGE_VALUE: f32 = 1.5;
pub const BUFF_FIRE_RATE_VALUE: f32 = 0.14;
pub const BUFF_BURST_INTERVAL: f32 = 3.5;
pub const BUFF_PIERCE_VALUE: i32 = 1;
pub const BUFF_STAGGER_ENABLED: bool = true;

pub const BASE_DAMAGE_VALUE: f32 = 1.0;
pub const DAMAGE_UPGRADE_APPLIES_TO_DRONES: bool = true;

pub const BASE_FIRE_RATE_VALUE: f32 = 0.18;
pub const FIRE_RATE_UPGRADE_APPLIES_TO_DRONES: bool = true;

pub const STAGGER_KNOCKBACK_PX: f32 = 12.0;

pub const MAX_ATTACHED_DRONES: usize = 2;
/// Base fire interval for attached drones (seconds). Upgraded by FIRE_RATE_UPGRADE_APPLIES_TO_DRONES.
pub const DRONE_FIRE_RATE: f32 = 0.18;
pub const DRONE_WIDTH: f32 = 24.0;
pub const DRONE_HEIGHT: f32 = 8.0;
pub const DRONE_REMOTE_WIDTH: f32 = 24.0;
pub const DRONE_REMOTE_HEIGHT: f32 = 10.0;
/// Y offsets from player.y for each attached drone slot (negative = above player).
pub const DRONE_Y_OFFSETS: [f32; 2] = [-10.0, 18.0];
/// Extra pixel gap between the player/drone stack and the lane boundaries.
pub const PLAYER_LANE_PADDING: f32 = 2.0;

pub const EXPLOSIVE_SHIELD_CLEAR_DISTANCE: f32 = 80.0;

/// Parallax background scroll speeds (px/s) for each layer (back, stars, props).
pub const BG_PARALLAX_SPEED_BACK: f32 = 0.0;
pub const BG_PARALLAX_SPEED_STARS: f32 = 8.0;
pub const BG_PARALLAX_SPEED_PROPS: f32 = 0.0;

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

    pub simultaneous_breach_window: Option<f32>,
    pub explosive_micro_stall: Option<f32>,
    pub re_breach_cooldown: Option<f32>,
    pub windup_time_small: Option<f32>,
    pub windup_time_medium: Option<f32>,
    pub windup_time_heavy: Option<f32>,
    pub windup_time_large: Option<f32>,
    pub windup_time_elite: Option<f32>,

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

    pub buff_damage_duration: Option<f32>,
    pub buff_fire_rate_duration: Option<f32>,
    pub buff_burst_duration: Option<f32>,
    pub buff_pierce_duration: Option<f32>,
    pub buff_stagger_duration: Option<f32>,

    pub buff_damage_value: Option<f32>,
    pub buff_fire_rate_value: Option<f32>,
    pub buff_burst_interval: Option<f32>,
    pub buff_pierce_value: Option<i32>,
    pub buff_stagger_enabled: Option<bool>,

    pub damage_upgrade_applies_to_drones: Option<bool>,

    pub fire_rate_upgrade_applies_to_drones: Option<bool>,

    pub bg_parallax_speed_back: Option<f32>,
    pub bg_parallax_speed_stars: Option<f32>,
    pub bg_parallax_speed_props: Option<f32>,

    // Debug flags
    pub explosive_shield_clear_distance: Option<f32>,

    pub debug_all_enemies: Option<bool>,
    pub debug_log_gameplay: Option<bool>,
    pub debug_log_file: Option<String>,
    pub debug_force_orb: Option<String>,
    pub debug_force_enemy: Option<String>,
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

    pub simultaneous_breach_window: f32,
    pub explosive_micro_stall: f32,
    pub re_breach_cooldown: f32,
    pub windup_time_small: f32,
    pub windup_time_medium: f32,
    pub windup_time_heavy: f32,
    pub windup_time_large: f32,
    pub windup_time_elite: f32,

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

    pub buff_damage_duration: f32,
    pub buff_fire_rate_duration: f32,
    pub buff_burst_duration: f32,
    pub buff_pierce_duration: f32,
    pub buff_stagger_duration: f32,

    pub buff_damage_value: f32,
    pub buff_fire_rate_value: f32,
    pub buff_burst_interval: f32,
    pub buff_pierce_value: i32,
    pub buff_stagger_enabled: bool,

    pub damage_upgrade_applies_to_drones: bool,

    pub fire_rate_upgrade_applies_to_drones: bool,

    pub bg_parallax_speed_back: f32,
    pub bg_parallax_speed_stars: f32,
    pub bg_parallax_speed_props: f32,

    /// Debug: spawn all enemy kinds from the start (bypasses intro timers).
    pub debug_all_enemies: bool,
    pub debug_log_gameplay: bool,
    pub debug_log_file: String,
    pub explosive_shield_clear_distance: f32,

    /// Debug: if Some, only this orb type spawns (bypasses level gates).
    pub debug_force_orb: Option<OrbType>,
    /// Debug: if Some, only this enemy kind spawns (bypasses intro timers and random selection).
    pub debug_force_enemy: Option<EnemyKind>,
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

            simultaneous_breach_window: rt
                .simultaneous_breach_window
                .unwrap_or(SIMULTANEOUS_BREACH_WINDOW),
            explosive_micro_stall: rt.explosive_micro_stall.unwrap_or(EXPLOSIVE_MICRO_STALL),
            re_breach_cooldown: rt.re_breach_cooldown.unwrap_or(RE_BREACH_COOLDOWN),
            windup_time_small: rt.windup_time_small.unwrap_or(WINDUP_TIME_SMALL),
            windup_time_medium: rt.windup_time_medium.unwrap_or(WINDUP_TIME_MEDIUM),
            windup_time_heavy: rt.windup_time_heavy.unwrap_or(WINDUP_TIME_HEAVY),
            windup_time_large: rt.windup_time_large.unwrap_or(WINDUP_TIME_LARGE),
            windup_time_elite: rt.windup_time_elite.unwrap_or(WINDUP_TIME_ELITE),

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

            buff_damage_duration: rt.buff_damage_duration.unwrap_or(BUFF_DAMAGE_DURATION),
            buff_fire_rate_duration: rt
                .buff_fire_rate_duration
                .unwrap_or(BUFF_FIRE_RATE_DURATION),
            buff_burst_duration: rt.buff_burst_duration.unwrap_or(BUFF_BURST_DURATION),
            buff_pierce_duration: rt.buff_pierce_duration.unwrap_or(BUFF_PIERCE_DURATION),
            buff_stagger_duration: rt.buff_stagger_duration.unwrap_or(BUFF_STAGGER_DURATION),

            buff_damage_value: rt.buff_damage_value.unwrap_or(BUFF_DAMAGE_VALUE),
            buff_fire_rate_value: rt.buff_fire_rate_value.unwrap_or(BUFF_FIRE_RATE_VALUE),
            buff_burst_interval: rt.buff_burst_interval.unwrap_or(BUFF_BURST_INTERVAL),
            buff_pierce_value: rt.buff_pierce_value.unwrap_or(BUFF_PIERCE_VALUE),
            buff_stagger_enabled: rt.buff_stagger_enabled.unwrap_or(BUFF_STAGGER_ENABLED),

            damage_upgrade_applies_to_drones: rt
                .damage_upgrade_applies_to_drones
                .unwrap_or(DAMAGE_UPGRADE_APPLIES_TO_DRONES),

            fire_rate_upgrade_applies_to_drones: rt
                .fire_rate_upgrade_applies_to_drones
                .unwrap_or(FIRE_RATE_UPGRADE_APPLIES_TO_DRONES),

            explosive_shield_clear_distance: rt
                .explosive_shield_clear_distance
                .unwrap_or(EXPLOSIVE_SHIELD_CLEAR_DISTANCE),

            bg_parallax_speed_back: rt.bg_parallax_speed_back.unwrap_or(BG_PARALLAX_SPEED_BACK),
            bg_parallax_speed_stars: rt
                .bg_parallax_speed_stars
                .unwrap_or(BG_PARALLAX_SPEED_STARS),
            bg_parallax_speed_props: rt
                .bg_parallax_speed_props
                .unwrap_or(BG_PARALLAX_SPEED_PROPS),

            debug_all_enemies: rt.debug_all_enemies.unwrap_or(false),
            debug_log_gameplay: rt.debug_log_gameplay.unwrap_or(DEBUG_LOG_GAMEPLAY),
            debug_log_file: rt
                .debug_log_file
                .unwrap_or_else(|| DEBUG_LOG_FILE.to_string()),
            debug_force_orb: rt.debug_force_orb.as_deref().and_then(|s| {
                match s.to_lowercase().as_str() {
                    "burst" => Some(OrbType::Burst),
                    "damage" => Some(OrbType::Damage),
                    "shield" => Some(OrbType::Shield),
                    "drone" => Some(OrbType::Drone),
                    "firerate" => Some(OrbType::FireRate),
                    "pierce" => Some(OrbType::Pierce),
                    "stagger" => Some(OrbType::Stagger),
                    "explosive" => Some(OrbType::Explosive),
                    "droneremote" => Some(OrbType::DroneRemote),
                    _ => None,
                }
            }),
            debug_force_enemy: rt.debug_force_enemy.as_deref().and_then(|s| {
                match s.to_lowercase().as_str() {
                    "small" => Some(EnemyKind::Small),
                    "medium" => Some(EnemyKind::Medium),
                    "heavy" => Some(EnemyKind::Heavy),
                    "large" => Some(EnemyKind::Large),
                    _ => None,
                }
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// load_runtime_config
// ---------------------------------------------------------------------------

/// Config values baked in at compile time from `config.toml`.
/// This ensures release builds use the same tuned values as the developer's local setup.
const BAKED_CONFIG: &str = include_str!("../config.toml");

fn baked_default() -> RuntimeConfig {
    toml::from_str::<RuntimeConfig>(BAKED_CONFIG)
        .expect("[config] config.toml embedded at compile time failed to parse")
}

pub fn load_runtime_config() -> RuntimeConfig {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(text) = std::fs::read_to_string("config.toml") {
            match toml::from_str::<RuntimeConfig>(&text) {
                Ok(cfg) => return cfg,
                Err(e) => panic!(
                    "[config] Failed to parse config.toml: {e}\nFix the file or delete it to regenerate defaults."
                ),
            }
        }
    }
    baked_default()
}

// ---------------------------------------------------------------------------
// save_default_config_if_missing
// ---------------------------------------------------------------------------

pub fn save_default_config_if_missing() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if std::fs::metadata("config.toml").is_ok() {
            return;
        }
        let default = RuntimeConfig::default();
        match toml::to_string_pretty(&default) {
            Ok(text) => {
                if let Err(e) = std::fs::write("config.toml", text) {
                    eprintln!("[config] Could not write config.toml: {e}");
                }
            }
            Err(e) => {
                eprintln!("[config] Could not serialize default config: {e}");
            }
        }
    }
}
