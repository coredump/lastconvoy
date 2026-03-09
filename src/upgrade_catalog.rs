// Permanent upgrade catalog: TOML-driven definitions and resolved gameplay modifiers.
// serde, toml, std::collections::HashMap
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const BAKED_UPGRADES: &str = include_str!("../upgrades.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    StartingShields,
    StartingDrones,
    OrbDropFrequency,
    StartingDamageLevel,
    MaxShieldCapBonus,
    ProjectileSpeedBonus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeDef {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub max_level: u32,
    pub cost_per_level: Vec<u32>,
    pub effect_type: EffectType,
    pub effect_per_level: Vec<f32>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeCatalog {
    pub upgrade: Vec<UpgradeDef>,
}

pub struct ResolvedUpgrades {
    pub extra_starting_shields: u32,
    pub extra_starting_drones: u32,
    pub orb_interval_reduction: f32,
    pub start_with_damage_buff: bool,
    pub shield_cap_bonus: u32,
    pub projectile_speed_bonus: f32,
}

impl UpgradeCatalog {
    pub fn resolve(&self, levels: &HashMap<String, u32>) -> ResolvedUpgrades {
        let mut result = ResolvedUpgrades {
            extra_starting_shields: 0,
            extra_starting_drones: 0,
            orb_interval_reduction: 0.0,
            start_with_damage_buff: false,
            shield_cap_bonus: 0,
            projectile_speed_bonus: 0.0,
        };
        for def in &self.upgrade {
            let level = *levels.get(&def.id).unwrap_or(&0);
            if level == 0 {
                continue;
            }
            let idx = (level as usize - 1).min(def.effect_per_level.len() - 1);
            let effect = def.effect_per_level[idx];
            match def.effect_type {
                EffectType::StartingShields => result.extra_starting_shields = effect as u32,
                EffectType::StartingDrones => result.extra_starting_drones = effect as u32,
                EffectType::OrbDropFrequency => result.orb_interval_reduction = effect,
                EffectType::StartingDamageLevel => result.start_with_damage_buff = effect > 0.0,
                EffectType::MaxShieldCapBonus => result.shield_cap_bonus = effect as u32,
                EffectType::ProjectileSpeedBonus => result.projectile_speed_bonus = effect,
            }
        }
        result
    }
}

pub fn load_upgrade_catalog() -> UpgradeCatalog {
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(content) = std::fs::read_to_string("upgrades.toml")
        && let Ok(catalog) = toml::from_str::<UpgradeCatalog>(&content)
    {
        return catalog;
    }
    toml::from_str::<UpgradeCatalog>(BAKED_UPGRADES).expect("baked upgrades.toml is valid")
}
