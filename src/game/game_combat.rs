// Combat logic: firing, projectile updates, collision, stagger, boundary, drone fire.
// super::GameState, crate::config, crate::enemy, crate::projectile, crate::shield
use crate::config::{
    BASE_DAMAGE_VALUE, BOUNDARY_X, DRONE_FIRE_RATE, DRONE_HEIGHT, DRONE_REMOTE_HEIGHT,
    DRONE_REMOTE_WIDTH, DRONE_Y_OFFSETS, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, PLAYER_WIDTH,
    PRE_BOUNDARY_STOP_OFFSET, PROJECTILE_H, PROJECTILE_W, SCREEN_W, SHAKE_DURATION,
    SHAKE_INTENSITY, SHIELD_FLASH_COLOR, SHIELD_FLASH_COOLDOWN, SHIELD_FLASH_DURATION,
    STAGGER_KNOCKBACK_PX,
};
use crate::enemy::{EnemyKind, EnemyState};
use crate::projectile::{Projectile, ProjectileSource};
use crate::shield::ShieldHitResult;

use super::{GameState, aabb_overlap};

impl GameState {
    pub(super) fn take_player_damage(&mut self) {
        self.player.shake.trigger(SHAKE_INTENSITY, SHAKE_DURATION);
        self.screen_flash.trigger(
            SHIELD_FLASH_COLOR,
            SHIELD_FLASH_DURATION,
            SHIELD_FLASH_COOLDOWN,
        );
        match self.shields.take_hit() {
            ShieldHitResult::NoShield => {
                if !self.game_over {
                    self.log_run_end("death");
                    let record = crate::save::RunRecord {
                        run_id: self.run_id,
                        run_time: self.run_time,
                        kills: self.kills_total,
                        breaches: self.breaches_total,
                        furthest_biome: self.furthest_biome,
                        loop_count: self.loop_count,
                        orbs_collected: self.orbs_collected.clone(),
                        timestamp: crate::save::current_timestamp(),
                    };
                    crate::save::record_run(&mut self.save, record);
                    crate::save::write_save(&self.save);
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

    pub(super) fn trigger_explosive_shield(&mut self) {
        let clear_distance = self.config.explosive_shield_clear_distance;
        let zone_right = BOUNDARY_X + clear_distance;
        let lane_top = ENEMY_LANE_TOP as f32;
        let lane_bottom = ENEMY_LANE_BOTTOM as f32;

        let mut stagger_targets: Vec<usize> = Vec::new();
        for (i, e) in self.enemies.iter_mut().enumerate() {
            if e.x < zone_right
                && e.x + e.width > BOUNDARY_X
                && e.y + e.height > lane_top
                && e.y < lane_bottom
            {
                match e.kind {
                    EnemyKind::Large | EnemyKind::XL | EnemyKind::Boss1 => {
                        stagger_targets.push(i);
                    }
                    _ => {
                        e.hp = 0;
                    }
                }
            }
        }

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
        if self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
        }
        self.boundary_ctrl.stall_timer = self.config.explosive_micro_stall;

        self.dlog(&format!(
            "EXPLOSIVE_TRIGGER zone_x0={:.0} zone_x1={:.0} lane_y0={} lane_y1={}",
            BOUNDARY_X, zone_right, ENEMY_LANE_TOP, ENEMY_LANE_BOTTOM
        ));
    }

    pub(super) fn update_firing(&mut self, dt: f32) {
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

        if self.player.should_fire() {
            let proj_x = self.player.x + self.player.width;
            let proj_y = self.player.y + (self.player.height - PROJECTILE_H) / 2.0;
            let speed = self.config.projectile_speed + self.projectile_speed_bonus;
            let pierce = self.current_pierce();
            if self.burst_ready {
                self.burst_ready = false;
                let spread_vy = speed * 0.105;
                let mut up =
                    Projectile::new(proj_x, proj_y, speed, ProjectileSource::Player, pierce);
                up.vy = -spread_vy;
                self.projectiles.push(up);
                let mut down =
                    Projectile::new(proj_x, proj_y, speed, ProjectileSource::Player, pierce);
                down.vy = spread_vy;
                self.projectiles.push(down);
            }
            self.projectiles.push(Projectile::new(
                proj_x,
                proj_y,
                speed,
                ProjectileSource::Player,
                pierce,
            ));
        }
    }

    pub(super) fn update_projectiles(&mut self, dt: f32) {
        for p in &mut self.projectiles {
            p.update(dt);
        }
        for p in &mut self.projectiles {
            if p.alive && Self::projectile_hits_shot_barrier(p) {
                p.alive = false;
            }
        }
    }

    pub(super) fn update_proj_enemy_hits(&mut self) {
        let mut kill_logs: Vec<String> = Vec::new();
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
            let proj_dmg = proj_dmg_base.round() as i32;
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
                            "KILL {:?} hp_max={} speed={:.1} shielded={} shield_hp={} dmg_total={} shots={} dmg_per_shot={:.2} source={:?}",
                            e.kind, e.max_hp, e.speed, e.shielded, e.shield_hp.max(0),
                            e.damage_taken, e.shots_taken, dps, p.source
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
            if self.enemies[idx].x + self.enemies[idx].width > SCREEN_W as f32 {
                self.enemies[idx].x = SCREEN_W as f32 - self.enemies[idx].width;
            }
        }
        if self.boundary_ctrl.breach_locked && self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
        }
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
                    if re_y >= left_y + left_h || re_y + re_h <= left_y {
                        continue;
                    }
                    if re_x < left_right {
                        self.enemies[right_idx].x = left_right;
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
    }

    pub(super) fn cleanup_dead_enemies(&mut self) {
        self.boundary_ctrl
            .breach_group
            .retain(|id| self.enemies.iter().any(|e| e.id == *id && !e.is_dead()));
        if self.boundary_ctrl.breach_group.is_empty() && self.boundary_ctrl.breach_locked {
            self.boundary_ctrl.breach_locked = false;
        }

        let exp_hw = self.explosion_sprite.tile_w as f32 * 0.5;
        let exp_hh = self.explosion_sprite.tile_h as f32 * 0.5;
        let new_explosions: Vec<super::Explosion> = self
            .enemies
            .iter()
            .filter(|e| e.is_dead() && !e.shake.is_active())
            .map(|e| super::Explosion {
                x: e.x + e.width * 0.5 - exp_hw,
                y: e.y + e.height * 0.5 - exp_hh,
                timer: 0.0,
            })
            .collect();
        self.explosions.extend(new_explosions);
        self.enemies.retain(|e| !e.is_dead() || e.shake.is_active());
    }

    pub(super) fn update_boundary(&mut self, dt: f32) {
        if self.boundary_ctrl.stall_timer > 0.0 {
            self.boundary_ctrl.stall_timer = (self.boundary_ctrl.stall_timer - dt).max(0.0);
        }
        let stalling = self.boundary_ctrl.stall_timer > 0.0;
        if self.boundary_ctrl.rebreach_cooldown > 0.0 {
            self.boundary_ctrl.rebreach_cooldown =
                (self.boundary_ctrl.rebreach_cooldown - dt).max(0.0);
        }

        for e in &mut self.enemies {
            if stalling {
                e.shake.update(dt);
            } else {
                let prev_x = e.x;
                e.update(dt);
                if self.boundary_ctrl.breach_locked && e.state == EnemyState::Moving {
                    let stop_x = BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET;
                    e.x = e.x.max(stop_x.min(prev_x));
                }
            }
        }

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
                let stop_x = if self.boundary_ctrl.breach_locked {
                    BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET
                } else {
                    BOUNDARY_X
                };
                self.enemies[i].x = stop_x.max(self.enemies[i].x);
            }
        }

        let mut resolved_ids: Vec<u64> = Vec::new();
        for e in &mut self.enemies {
            if e.state == EnemyState::Breaching {
                e.windup_elapsed += dt;
                if e.windup_elapsed >= e.windup_time {
                    resolved_ids.push(e.id);
                }
            }
        }

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

        if self.boundary_ctrl.breach_locked && self.boundary_ctrl.breach_group.is_empty() {
            self.boundary_ctrl.breach_locked = false;
            if had_resolution {
                self.boundary_ctrl.rebreach_cooldown = self.config.re_breach_cooldown;
            }
        }

        self.enemies
            .retain(|e| !e.is_off_screen() && (!e.is_dead() || e.shake.is_active()));

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

        if self.boundary_ctrl.rebreach_cooldown > 0.0 {
            for e in &mut self.enemies {
                if e.state == EnemyState::Moving && e.x < BOUNDARY_X {
                    e.x = BOUNDARY_X;
                }
            }
        }
    }

    pub(super) fn update_proj_orb_hits(&mut self) {
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
                    if o.phase == crate::orb::OrbPhase::Inactive {
                        o.hit_this_frame = true;
                    }
                    p.alive = false;
                    break;
                }
            }
        }
    }

    pub(super) fn update_drones(&mut self, dt: f32) {
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
                    self.config.projectile_speed + self.projectile_speed_bonus,
                    ProjectileSource::Drone,
                    drone_pierce,
                ));
            }
        }
        self.projectiles.extend(drone_shots);

        if self.orb_activated_this_frame {
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
                        self.config.projectile_speed + self.projectile_speed_bonus,
                        ProjectileSource::RemoteDrone,
                        0,
                    ));
                }
            }
            self.projectiles.extend(rd_shots);
        }
    }
}
