// Cross-run persistent state: save/load, run history, and meta/story placeholders.
// serde_json for serialization; std::fs on native, quad-storage on WASM.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 2;
const SAVE_FILE: &str = "lastconvoy_save.json";
const SAVE_KEY: &str = "lastconvoy_save";
const MAX_RUN_HISTORY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveData {
    pub version: u32,
    pub total_runs: u32,
    pub best_run_time: f32,
    pub best_kills: u32,
    pub best_loop_count: u32,
    pub best_biome_reached: u32,
    pub run_history: Vec<RunRecord>,
    pub lifetime_kills: u32,
    pub lifetime_breaches: u32,
    pub lifetime_play_time: f32,
    pub lifetime_orbs_collected: OrbStats,
    pub meta_points: u32,
    pub meta_points_lifetime: u32,
    pub permanent_upgrades: PermanentUpgrades,
    pub story_progress: StoryProgress,
    pub tutorial_dismissed: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            total_runs: 0,
            best_run_time: 0.0,
            best_kills: 0,
            best_loop_count: 0,
            best_biome_reached: 0,
            run_history: Vec::new(),
            lifetime_kills: 0,
            lifetime_breaches: 0,
            lifetime_play_time: 0.0,
            lifetime_orbs_collected: OrbStats::default(),
            meta_points: 0,
            meta_points_lifetime: 0,
            permanent_upgrades: PermanentUpgrades::default(),
            story_progress: StoryProgress::default(),
            tutorial_dismissed: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RunRecord {
    pub run_id: u32,
    pub run_time: f32,
    pub kills: u32,
    pub breaches: u32,
    pub furthest_biome: u32,
    pub loop_count: u32,
    pub orbs_collected: OrbStats,
    pub timestamp: u64,
}

impl Default for RunRecord {
    fn default() -> Self {
        Self {
            run_id: 0,
            run_time: 0.0,
            kills: 0,
            breaches: 0,
            furthest_biome: 0,
            loop_count: 0,
            orbs_collected: OrbStats::default(),
            timestamp: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OrbStats {
    pub burst: u32,
    pub damage: u32,
    pub shield: u32,
    pub drone: u32,
    pub drone_remote: u32,
    pub explosive: u32,
    pub fire_rate: u32,
    pub pierce: u32,
    pub stagger: u32,
}

impl OrbStats {
    pub fn add(&mut self, other: &OrbStats) {
        self.burst += other.burst;
        self.damage += other.damage;
        self.shield += other.shield;
        self.drone += other.drone;
        self.drone_remote += other.drone_remote;
        self.explosive += other.explosive;
        self.fire_rate += other.fire_rate;
        self.pierce += other.pierce;
        self.stagger += other.stagger;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PermanentUpgrades {
    pub levels: HashMap<String, u32>,
    #[serde(skip_serializing, default)]
    pub starting_shields: u32,
    #[serde(skip_serializing, default)]
    pub starting_drones: u32,
    #[serde(skip_serializing, default)]
    pub orb_drop_frequency: u32,
}

impl PermanentUpgrades {
    pub fn get_level(&self, id: &str) -> u32 {
        *self.levels.get(id).unwrap_or(&0)
    }

    pub fn set_level(&mut self, id: &str, level: u32) {
        self.levels.insert(id.to_string(), level);
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StoryProgress {
    pub seen_beats: Vec<String>,
    pub unlocked_characters: Vec<String>,
}

pub fn load_save() -> SaveData {
    let json = platform_read();
    if json.is_empty() {
        return SaveData::default();
    }
    match serde_json::from_str::<SaveData>(&json) {
        Ok(mut data) => {
            migrate(&mut data);
            data
        }
        Err(e) => {
            eprintln!("[save] Failed to parse save: {e}");
            SaveData::default()
        }
    }
}

pub fn write_save(data: &SaveData) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => platform_write(&json),
        Err(e) => eprintln!("[save] Failed to serialize save: {e}"),
    }
}

pub fn record_run(save: &mut SaveData, record: RunRecord) {
    save.total_runs += 1;
    save.lifetime_kills += record.kills;
    save.lifetime_breaches += record.breaches;
    save.lifetime_play_time += record.run_time;
    save.lifetime_orbs_collected.add(&record.orbs_collected);
    if record.run_time > save.best_run_time {
        save.best_run_time = record.run_time;
    }
    if record.kills > save.best_kills {
        save.best_kills = record.kills;
    }
    if record.loop_count > save.best_loop_count {
        save.best_loop_count = record.loop_count;
    }
    if record.furthest_biome > save.best_biome_reached {
        save.best_biome_reached = record.furthest_biome;
    }
    save.run_history.insert(0, record);
    save.run_history.truncate(MAX_RUN_HISTORY);
}

pub fn current_timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

fn migrate(data: &mut SaveData) {
    if data.version < 2 {
        let pu = &mut data.permanent_upgrades;
        if pu.starting_shields > 0 {
            pu.levels
                .insert("starting_shields".to_string(), pu.starting_shields);
        }
        if pu.starting_drones > 0 {
            pu.levels
                .insert("starting_drones".to_string(), pu.starting_drones);
        }
        if pu.orb_drop_frequency > 0 {
            pu.levels
                .insert("orb_drop_frequency".to_string(), pu.orb_drop_frequency);
        }
    }
    data.version = SAVE_VERSION;
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_read() -> String {
    std::fs::read_to_string(SAVE_FILE).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_write(json: &str) {
    let tmp = format!("{SAVE_FILE}.tmp");
    if let Err(e) = std::fs::write(&tmp, json) {
        eprintln!("[save] Failed to write {tmp}: {e}");
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, SAVE_FILE) {
        eprintln!("[save] Failed to rename {tmp} -> {SAVE_FILE}: {e}");
    }
}

#[cfg(target_arch = "wasm32")]
fn platform_read() -> String {
    quad_storage::STORAGE
        .lock()
        .ok()
        .and_then(|s| s.get(SAVE_KEY))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn platform_write(json: &str) {
    if let Ok(mut s) = quad_storage::STORAGE.lock() {
        s.set(SAVE_KEY, json);
    }
}
