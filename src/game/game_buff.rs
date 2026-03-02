// Buff timers, buff queries, damage/fire-rate/pierce accessors, balance telemetry.
// super::GameState, crate::config
use crate::config::BASE_DAMAGE_VALUE;

use super::GameState;

impl GameState {
    pub(super) fn damage_buff_active(&self) -> bool {
        self.damage_buff_t > 0.0
    }

    pub(super) fn fire_rate_buff_active(&self) -> bool {
        self.fire_rate_buff_t > 0.0
    }

    pub(super) fn burst_buff_active(&self) -> bool {
        self.burst_buff_t > 0.0
    }

    pub(super) fn pierce_buff_active(&self) -> bool {
        self.pierce_buff_t > 0.0
    }

    pub(super) fn stagger_buff_active(&self) -> bool {
        self.stagger_buff_t > 0.0 && self.config.buff_stagger_enabled
    }

    pub(super) fn current_damage(&self) -> f32 {
        if self.damage_buff_active() {
            self.config.buff_damage_value
        } else {
            BASE_DAMAGE_VALUE
        }
    }

    pub(super) fn current_fire_rate(&self) -> f32 {
        if self.fire_rate_buff_active() {
            self.config.buff_fire_rate_value
        } else {
            self.config.player_fire_rate
        }
    }

    pub(super) fn current_pierce(&self) -> i32 {
        if self.pierce_buff_active() {
            self.config.buff_pierce_value
        } else {
            0
        }
    }

    pub(super) fn tick_buff_timers(&mut self, dt: f32) {
        self.damage_buff_t = (self.damage_buff_t - dt).max(0.0);
        self.fire_rate_buff_t = (self.fire_rate_buff_t - dt).max(0.0);
        self.burst_buff_t = (self.burst_buff_t - dt).max(0.0);
        self.pierce_buff_t = (self.pierce_buff_t - dt).max(0.0);
        self.stagger_buff_t = (self.stagger_buff_t - dt).max(0.0);
    }

    pub(super) fn dps_estimate(&self) -> f32 {
        let dmg = self.current_damage();
        let fire_rate = self.current_fire_rate();
        dmg / fire_rate
    }

    pub(super) fn large_ttk(&self) -> f32 {
        let base_hp = self.config.enemy_large_hp as f32;
        let hp_mult =
            1.0 + self.config.enemy_hp_scale * self.run_time * self.config.hp_scale_large_mult;
        let large_hp = (base_hp * hp_mult).round().max(1.0);
        let dmg = self.current_damage();
        let fire_rate = self.current_fire_rate();
        large_hp * fire_rate / dmg
    }

    pub(super) fn log_balance_snapshot(&mut self) {
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
}
