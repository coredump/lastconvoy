// Orb spawning, orb movement/activation, orb collection, floating upgrade text.
// super::GameState, crate::orb, crate::drone, crate::shield
use crate::config::{
    BOUNDARY_X, DRONE_REMOTE_HEIGHT, DRONE_Y_OFFSETS, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP,
    MAX_ATTACHED_DRONES, ORB_H, ORB_W, SCREEN_W,
};
use crate::drone::{Drone, RemoteDrone, RemoteDroneLane};
use crate::orb::{Orb, OrbPhase, OrbType};
use macroquad::prelude::{Color, rand};

use super::{FLOATING_TEXT_TTL, FLOATING_TEXT_VY, FloatingText, GameState, aabb_overlap};

impl GameState {
    pub(super) fn update_orb_spawning(&mut self, dt: f32) {
        self.orb_spawn_timer -= dt;
        if self.orb_spawn_timer > 0.0 {
            return;
        }
        self.orb_spawn_timer = self.config.orb_spawn_interval;
        if self.orbs.len() >= self.config.max_active_orbs {
            return;
        }
        let shields_full = self.shields.count() >= crate::shield::MAX_SHIELD_SEGMENTS;
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
        let drone_remaining = (MAX_ATTACHED_DRONES.saturating_sub(self.drones.len())) as u32;
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
        if pool.is_empty() {
            return;
        }
        let total: u32 = pool.iter().map(|(_, w)| w).sum();
        let roll_orb_type = |pool: &[(OrbType, u32)], total: u32| -> OrbType {
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
            let top_type = roll_orb_type(&pool, total);
            let bottom_type = roll_orb_type(&pool, total);
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
                roll_orb_type(&pool, total),
            ));
        }
    }

    pub(super) fn update_orbs(&mut self, dt: f32) {
        self.orb_activated_this_frame = false;
        for o in &mut self.orbs {
            let was_inactive = o.phase == OrbPhase::Inactive;
            o.update(dt);
            if was_inactive && o.phase == OrbPhase::Active {
                self.orb_activated_this_frame = true;
            }
        }
    }

    pub(super) fn update_orb_collection(&mut self) {
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
    }

    pub(super) fn spawn_upgrade_floating_text(&mut self, tag: &str, x: f32, y: f32) {
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

    pub(super) fn update_floating_texts(&mut self, dt: f32) {
        for t in &mut self.floating_texts {
            t.y += t.vy * dt;
            t.life = (t.life - dt).max(0.0);
        }
        self.floating_texts.retain(|t| t.life > 0.0);
    }
}
