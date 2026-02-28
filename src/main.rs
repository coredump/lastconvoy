// Phase 1: stub modules are intentionally incomplete — suppress dead_code warnings.
#![allow(dead_code)]

use macroquad::prelude::*;

mod boundary;
mod config;
mod debug_log;
mod drone;
mod elite;
mod enemy;
mod game;
mod input;
mod orb;
mod player;
mod projectile;
mod render;
mod shield;
mod sprite;
mod upgrade;

use config::{
    Config, SCREEN_H, SCREEN_W, WINDOW_SCALE, load_runtime_config, save_default_config_if_missing,
};
use game::GameState;
use render::RenderPipeline;
use sprite::Sprite;

fn window_conf() -> Conf {
    Conf {
        window_title: "LCDShootSystem".to_string(),
        window_width: (SCREEN_W * WINDOW_SCALE) as i32,
        window_height: (SCREEN_H * WINDOW_SCALE) as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    save_default_config_if_missing();
    let runtime_cfg = load_runtime_config();
    let config = Config::from_runtime(runtime_cfg);

    let player_sprite = Sprite::from_json("assets/sprites/player/player.json")
        .await
        .expect("Failed to load player sprite");
    let enemy_small_sprite = Sprite::from_json("assets/sprites/enemies/small.json")
        .await
        .expect("Failed to load enemy_small sprite");
    let enemy_medium_sprite = Sprite::from_json("assets/sprites/enemies/medium.json")
        .await
        .expect("Failed to load enemy_medium sprite");
    let enemy_heavy_sprite = Sprite::from_json("assets/sprites/enemies/heavy.json")
        .await
        .expect("Failed to load enemy_heavy sprite");
    let enemy_large_sprite = Sprite::from_json("assets/sprites/enemies/large.json")
        .await
        .expect("Failed to load enemy_large sprite");
    let enemy_elite_sprite = Sprite::from_json("assets/sprites/enemies/elite.json")
        .await
        .expect("Failed to load enemy_elite sprite");
    let boundary_shield_sprite = Sprite::from_json("assets/sprites/objects/boundary_shield.json")
        .await
        .expect("Failed to load boundary_shield sprite");

    // Load one Sprite per OrbType, each pre-locked to its animation tag index.
    // Tag indices in upgrades.json: 0=damage, 4=stagger, 5=extradrone, 7=shield
    let mut orb_sprite_damage = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (damage)");
    orb_sprite_damage.set_animation(0);

    let mut orb_sprite_defense = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (defense)");
    orb_sprite_defense.set_animation(7);

    let mut orb_sprite_drone = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (drone)");
    orb_sprite_drone.set_animation(5);

    let mut orb_sprite_fire_rate = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (fire_rate)");
    orb_sprite_fire_rate.set_animation(1);

    let mut orb_sprite_burst = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (burst)");
    orb_sprite_burst.set_animation(2);

    let mut orb_sprite_pierce = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (pierce)");
    orb_sprite_pierce.set_animation(3);

    let mut orb_sprite_stagger = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite (stagger)");
    orb_sprite_stagger.set_animation(4);

    let mut state = GameState::new(
        config,
        player_sprite,
        enemy_small_sprite,
        enemy_medium_sprite,
        enemy_heavy_sprite,
        enemy_large_sprite,
        enemy_elite_sprite,
        boundary_shield_sprite,
        orb_sprite_damage,
        orb_sprite_defense,
        orb_sprite_drone,
        orb_sprite_fire_rate,
        orb_sprite_burst,
        orb_sprite_pierce,
        orb_sprite_stagger,
    );
    let pipeline = RenderPipeline::new();

    loop {
        let dt = get_frame_time();
        state.update(dt);
        pipeline.begin();
        state.draw();
        pipeline.end_and_blit();
        next_frame().await;
    }
}
