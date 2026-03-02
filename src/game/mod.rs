// Core game state, struct definitions, update orchestrator, and logging.
// macroquad, all entity modules
use macroquad::prelude::*;

use crate::config::{
    BIG_INJECT_BASE_INTERVAL, Config, ENEMY_LANE_BOTTOM, ENEMY_LANE_TOP, PLAYER_HEIGHT,
    PLAYER_WIDTH, PLAYER_X, PROJECTILE_H, SHOT_BARRIER_BOTTOM_Y, SHOT_BARRIER_GATE_X_MAX,
    SHOT_BARRIER_TOP_Y, SPAWN_TICK_INTERVAL, TOP_UPGRADE_LANE_BOTTOM, TOP_UPGRADE_LANE_TOP,
    UPGRADE_LANE_BOTTOM, UPGRADE_LANE_TOP,
};
use crate::drone::{Drone, RemoteDrone};
use crate::elite::EliteEvent;
use crate::enemy::Enemy;
use crate::input::InputState;
use crate::orb::Orb;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::shield::ShieldSystem;
use crate::sprite::Sprite;
use crate::text::BitmapFont;

mod game_buff;
mod game_combat;
mod game_draw;
mod game_orb;
mod game_spawn;

pub(super) const FLOATING_TEXT_TTL: f32 = 0.8;
pub(super) const FLOATING_TEXT_VY: f32 = -18.0;

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
    pub breach_group: Vec<u64>,
    pub breach_start_time: f32,
    pub breach_locked: bool,
    pub stall_timer: f32,
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

pub(super) const EXPLOSION_FRAME_DUR: f32 = 0.04;
pub(super) const EXPLOSION_FRAME_COUNT: u32 = 5;
pub(super) const EXPLOSION_TOTAL_DUR: f32 = EXPLOSION_FRAME_DUR * EXPLOSION_FRAME_COUNT as f32;

pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub timer: f32,
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
    pub upgrade_track_sprite: Sprite,
    pub top_bar_sprite: Sprite,
    pub bg_texture: Texture2D,
    pub bg_scroll_offsets: [f32; 3],
    pub shot_sprite: Sprite,
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
    pub at_title: bool,
    pub paused: bool,
    pub game_over: bool,
    pub kills_total: u32,
    pub breaches_total: u32,
    pub balance_log_timer: f32,
    pub debug_log: Option<crate::debug_log::DebugLog>,
    pub additive_material: Material,
    pub color_blend_material: Material,
    pub ui_font: BitmapFont,
    pub logo_font: BitmapFont,
    pub monogram_font: BitmapFont,
    pub logo_sprite: Sprite,
    pub floating_texts: Vec<FloatingText>,
    pub explosion_sprite: Sprite,
    pub explosions: Vec<Explosion>,
    pub(super) orb_activated_this_frame: bool,
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
        upgrade_track_sprite: Sprite,
        explosion_sprite: Sprite,
        top_bar_sprite: Sprite,
        bg_texture: Texture2D,
        ui_font: BitmapFont,
        logo_font: BitmapFont,
        monogram_font: BitmapFont,
        logo_sprite: Sprite,
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
            shot_sprite,
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
            upgrade_track_sprite,
            top_bar_sprite,
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
            at_title: true,
            paused: false,
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
            logo_font,
            monogram_font,
            logo_sprite,
            floating_texts: Vec::new(),
            explosion_sprite,
            explosions: Vec::new(),
            orb_activated_this_frame: false,
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
            color_blend_material: {
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
                            precision mediump float;
                            varying lowp vec4 color;
                            varying lowp vec2 uv;
                            uniform sampler2D Texture;

                            vec3 rgb2hsl(vec3 c) {
                                float maxc = max(c.r, max(c.g, c.b));
                                float minc = min(c.r, min(c.g, c.b));
                                float l = (maxc + minc) * 0.5;
                                float d = maxc - minc;
                                if (d < 0.0001) return vec3(0.0, 0.0, l);
                                float s = d / (1.0 - abs(2.0 * l - 1.0));
                                float h;
                                if (maxc == c.r)      h = mod((c.g - c.b) / d, 6.0);
                                else if (maxc == c.g) h = (c.b - c.r) / d + 2.0;
                                else                  h = (c.r - c.g) / d + 4.0;
                                h /= 6.0;
                                return vec3(h, s, l);
                            }

                            vec3 hsl2rgb(vec3 hsl) {
                                float h = hsl.x, s = hsl.y, l = hsl.z;
                                float c = (1.0 - abs(2.0 * l - 1.0)) * s;
                                float x = c * (1.0 - abs(mod(h * 6.0, 2.0) - 1.0));
                                float m = l - c * 0.5;
                                vec3 rgb;
                                float h6 = h * 6.0;
                                if      (h6 < 1.0) rgb = vec3(c, x, 0.0);
                                else if (h6 < 2.0) rgb = vec3(x, c, 0.0);
                                else if (h6 < 3.0) rgb = vec3(0.0, c, x);
                                else if (h6 < 4.0) rgb = vec3(0.0, x, c);
                                else if (h6 < 5.0) rgb = vec3(x, 0.0, c);
                                else               rgb = vec3(c, 0.0, x);
                                return rgb + m;
                            }

                            void main() {
                                vec4 texel = texture2D(Texture, uv);
                                vec3 hsl_base = rgb2hsl(texel.rgb);
                                vec3 hsl_tint = rgb2hsl(color.rgb);
                                vec3 blended = hsl2rgb(vec3(hsl_tint.x, hsl_tint.y, hsl_base.z));
                                gl_FragColor = vec4(blended, texel.a);
                            }"#,
                    },
                    MaterialParams::default(),
                )
                .expect("color_blend material")
            },
        }
    }

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
        self.at_title = false;
        self.paused = false;
        self.game_over = false;
        self.kills_total = 0;
        self.breaches_total = 0;
        self.balance_log_timer = 0.0;
        self.floating_texts.clear();
        self.explosions.clear();
        self.orb_activated_this_frame = false;
        self.log_run_start("restart");
    }

    pub(super) fn dlog(&mut self, msg: &str) {
        if let Some(log) = &mut self.debug_log {
            log.log(self.run_time, msg);
        }
    }

    pub(super) fn log_run_start(&mut self, source: &str) {
        self.dlog(&format!(
            "RUN_START run_id={} source={}",
            self.run_id, source
        ));
    }

    pub(super) fn log_run_end(&mut self, reason: &str) {
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

    pub fn update(&mut self, dt: f32) {
        if self.at_title {
            self.logo_sprite.update(dt);
            if get_keys_pressed().into_iter().next().is_some() {
                self.at_title = false;
                self.log_run_start("new_game");
            }
            return;
        }

        if self.game_over {
            if is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::R)
            {
                self.reset();
            }
            return;
        }

        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.paused = !self.paused;
        }
        if self.paused {
            return;
        }

        self.run_time += dt;
        self.update_floating_texts(dt);
        for exp in &mut self.explosions {
            exp.timer += dt;
        }
        self.explosions.retain(|e| e.timer < EXPLOSION_TOTAL_DUR);
        self.tick_buff_timers(dt);
        self.player.fire_rate = self.current_fire_rate();

        self.balance_log_timer -= dt;
        if self.balance_log_timer <= 0.0 {
            self.balance_log_timer = 30.0;
            self.log_balance_snapshot();
        }

        self.input.update(&self.config);

        let axis = self.input.axis;
        let has_top_drone = !self.drones.is_empty();
        let has_bottom_drone = self.drones.len() >= 2;
        self.player
            .update(axis, dt, has_top_drone, has_bottom_drone);

        self.update_firing(dt);
        self.update_projectiles(dt);
        self.update_proj_enemy_hits();
        self.cleanup_dead_enemies();

        self.spawn_ctrl.tick_accum += dt;
        while self.spawn_ctrl.tick_accum >= SPAWN_TICK_INTERVAL {
            self.spawn_ctrl.tick_accum -= SPAWN_TICK_INTERVAL;
            self.tick_spawn();
        }

        self.update_boundary(dt);
        self.update_orb_spawning(dt);
        self.update_proj_orb_hits();
        self.update_orbs(dt);
        self.update_orb_collection();
        self.update_drones(dt);

        let bg_speeds = [
            self.config.bg_parallax_speed_back,
            self.config.bg_parallax_speed_stars,
            self.config.bg_parallax_speed_props,
        ];
        for (off, &spd) in self.bg_scroll_offsets.iter_mut().zip(bg_speeds.iter()) {
            *off += spd * dt;
        }

        self.update_animations(dt);
    }

    pub(super) fn format_run_timer(&self) -> String {
        let total = self.run_time.max(0.0).floor() as u32;
        let minutes = total / 60;
        let seconds = total % 60;
        format!("{minutes:02}:{seconds:02}")
    }

    pub(super) fn upgrade_lane_mid_top(&self) -> f32 {
        (TOP_UPGRADE_LANE_TOP + TOP_UPGRADE_LANE_BOTTOM + 1) as f32 / 2.0
    }

    pub(super) fn upgrade_lane_mid_bottom(&self) -> f32 {
        (UPGRADE_LANE_TOP + UPGRADE_LANE_BOTTOM + 1) as f32 / 2.0
    }

    pub(super) fn projectile_hits_shot_barrier(p: &Projectile) -> bool {
        if p.x <= SHOT_BARRIER_GATE_X_MAX {
            return false;
        }
        let y0 = p.y;
        let y1 = p.y + PROJECTILE_H;
        let overlaps_top = y0 < SHOT_BARRIER_TOP_Y + 1.0 && y1 > SHOT_BARRIER_TOP_Y;
        let overlaps_bottom = y0 < SHOT_BARRIER_BOTTOM_Y + 1.0 && y1 > SHOT_BARRIER_BOTTOM_Y;
        overlaps_top || overlaps_bottom
    }

    fn update_animations(&mut self, dt: f32) {
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
        self.player_sprite.update(dt);
        self.enemy_small_sprite.update(dt);
        self.enemy_medium_sprite.update(dt);
        self.enemy_heavy_sprite.update(dt);
        self.enemy_large_sprite.update(dt);
        self.enemy_elite_sprite.update(dt);
        self.boundary_shield_sprite.update(dt);
        self.rail_wall_sprite.update(dt);
        self.upgrade_track_sprite.update(dt);
        self.shields.update(dt);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn aabb_overlap(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}
