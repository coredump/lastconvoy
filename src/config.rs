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
pub const PLAYER_STARTING_SHIELDS: u32 = 3;

// Enemy sizes (w, h)
pub const ENEMY_SMALL_W: f32 = 16.0;
pub const ENEMY_SMALL_H: f32 = 16.0;
pub const ENEMY_MEDIUM_W: f32 = 24.0;
pub const ENEMY_MEDIUM_H: f32 = 24.0;
pub const ENEMY_HEAVY_W: f32 = 32.0;
pub const ENEMY_HEAVY_H: f32 = 24.0;
pub const ENEMY_LARGE_W: f32 = 40.0;
pub const ENEMY_LARGE_H: f32 = 32.0;

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
pub const ENEMY_SPAWN_INTERVAL: f32 = 2.0;
pub const ORB_SPAWN_INTERVAL: f32 = 5.0;
pub const MAX_ACTIVE_ORBS: usize = 4;

// Boundary
pub const BOUNDARY_SLOT_COUNT: usize = 3;
pub const BOUNDARY_DAMAGE_TICK: f32 = 2.0; // seconds between boundary damage ticks

// Orb
pub const ORB_HP: i32 = 3;
pub const ORB_SPEED: f32 = 25.0;
pub const ORB_W: f32 = 12.0;
pub const ORB_H: f32 = 12.0;

// Elite / MiniBoss
pub const ELITE_INTERVAL: f32 = 60.0;
pub const ELITE_INTERVAL_JITTER: f32 = 10.0;
pub const MINIBOSS_INTERVAL: f32 = 180.0;
pub const MINIBOSS_INTERVAL_JITTER: f32 = 20.0;
pub const ELITE_HP: i32 = 20;
pub const MINIBOSS_HP: i32 = 30;
pub const ELITE_SPEED: f32 = 18.0;
pub const MINIBOSS_SPEED: f32 = 14.0;

// Scaling rates (fractional increase per second)
pub const SPAWN_RATE_SCALE: f32 = 0.002;
pub const ENEMY_HP_SCALE: f32 = 0.001;
pub const SHIELDED_FREQ_SCALE: f32 = 0.001;

// Medium/Large introduction times (seconds into run)
pub const MEDIUM_INTRO_TIME: f32 = 30.0;
pub const HEAVY_INTRO_TIME: f32 = 90.0;
pub const LARGE_INTRO_TIME: f32 = 150.0;

// ---------------------------------------------------------------------------
// RuntimeConfig — all fields Optional; RON file only needs overrides
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

    pub enemy_spawn_interval: Option<f32>,
    pub orb_spawn_interval: Option<f32>,
    pub max_active_orbs: Option<usize>,

    pub boundary_slot_count: Option<usize>,
    pub boundary_damage_tick: Option<f32>,

    pub orb_hp: Option<i32>,
    pub orb_speed: Option<f32>,

    pub elite_interval: Option<f32>,
    pub elite_interval_jitter: Option<f32>,
    pub miniboss_interval: Option<f32>,
    pub miniboss_interval_jitter: Option<f32>,
    pub elite_hp: Option<i32>,
    pub miniboss_hp: Option<i32>,

    pub spawn_rate_scale: Option<f32>,
    pub enemy_hp_scale: Option<f32>,
    pub shielded_freq_scale: Option<f32>,

    pub medium_intro_time: Option<f32>,
    pub heavy_intro_time: Option<f32>,
    pub large_intro_time: Option<f32>,

    pub rotate_input: Option<bool>,
    pub analog_deadzone: Option<f32>,

    pub projectile_speed: Option<f32>,
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

    pub enemy_spawn_interval: f32,
    pub orb_spawn_interval: f32,
    pub max_active_orbs: usize,

    pub boundary_slot_count: usize,
    pub boundary_damage_tick: f32,

    pub orb_hp: i32,
    pub orb_speed: f32,

    pub elite_interval: f32,
    pub elite_interval_jitter: f32,
    pub miniboss_interval: f32,
    pub miniboss_interval_jitter: f32,
    pub elite_hp: i32,
    pub miniboss_hp: i32,

    pub spawn_rate_scale: f32,
    pub enemy_hp_scale: f32,
    pub shielded_freq_scale: f32,

    pub medium_intro_time: f32,
    pub heavy_intro_time: f32,
    pub large_intro_time: f32,

    pub rotate_input: bool,
    pub analog_deadzone: f32,

    pub projectile_speed: f32,
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

            enemy_spawn_interval: rt.enemy_spawn_interval.unwrap_or(ENEMY_SPAWN_INTERVAL),
            orb_spawn_interval: rt.orb_spawn_interval.unwrap_or(ORB_SPAWN_INTERVAL),
            max_active_orbs: rt.max_active_orbs.unwrap_or(MAX_ACTIVE_ORBS),

            boundary_slot_count: rt.boundary_slot_count.unwrap_or(BOUNDARY_SLOT_COUNT),
            boundary_damage_tick: rt.boundary_damage_tick.unwrap_or(BOUNDARY_DAMAGE_TICK),

            orb_hp: rt.orb_hp.unwrap_or(ORB_HP),
            orb_speed: rt.orb_speed.unwrap_or(ORB_SPEED),

            elite_interval: rt.elite_interval.unwrap_or(ELITE_INTERVAL),
            elite_interval_jitter: rt.elite_interval_jitter.unwrap_or(ELITE_INTERVAL_JITTER),
            miniboss_interval: rt.miniboss_interval.unwrap_or(MINIBOSS_INTERVAL),
            miniboss_interval_jitter: rt
                .miniboss_interval_jitter
                .unwrap_or(MINIBOSS_INTERVAL_JITTER),
            elite_hp: rt.elite_hp.unwrap_or(ELITE_HP),
            miniboss_hp: rt.miniboss_hp.unwrap_or(MINIBOSS_HP),

            spawn_rate_scale: rt.spawn_rate_scale.unwrap_or(SPAWN_RATE_SCALE),
            enemy_hp_scale: rt.enemy_hp_scale.unwrap_or(ENEMY_HP_SCALE),
            shielded_freq_scale: rt.shielded_freq_scale.unwrap_or(SHIELDED_FREQ_SCALE),

            medium_intro_time: rt.medium_intro_time.unwrap_or(MEDIUM_INTRO_TIME),
            heavy_intro_time: rt.heavy_intro_time.unwrap_or(HEAVY_INTRO_TIME),
            large_intro_time: rt.large_intro_time.unwrap_or(LARGE_INTRO_TIME),

            rotate_input: rt.rotate_input.unwrap_or(ROTATE_INPUT),
            analog_deadzone: rt.analog_deadzone.unwrap_or(ANALOG_DEADZONE),

            projectile_speed: rt.projectile_speed.unwrap_or(PROJECTILE_SPEED),
        }
    }
}

// ---------------------------------------------------------------------------
// load_runtime_config
// ---------------------------------------------------------------------------

pub fn load_runtime_config() -> RuntimeConfig {
    match fs::read_to_string("config.ron") {
        Ok(text) => match ron::from_str::<RuntimeConfig>(&text) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("[config] Failed to parse config.ron: {e}. Using defaults.");
                RuntimeConfig::default()
            }
        },
        Err(_) => RuntimeConfig::default(),
    }
}

// ---------------------------------------------------------------------------
// save_default_config_if_missing
// ---------------------------------------------------------------------------

pub fn save_default_config_if_missing() {
    if fs::metadata("config.ron").is_ok() {
        return;
    }
    let default = RuntimeConfig::default();
    match ron::ser::to_string_pretty(&default, ron::ser::PrettyConfig::default()) {
        Ok(text) => {
            if let Err(e) = fs::write("config.ron", text) {
                eprintln!("[config] Could not write config.ron: {e}");
            }
        }
        Err(e) => {
            eprintln!("[config] Could not serialize default config: {e}");
        }
    }
}
