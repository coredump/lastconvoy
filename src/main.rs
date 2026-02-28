// Phase 1: stub modules are intentionally incomplete — suppress dead_code warnings.
#![allow(dead_code)]

use macroquad::prelude::*;

mod boundary;
mod config;
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
    let orb_sprite = Sprite::from_json("assets/sprites/objects/upgrades.json")
        .await
        .expect("Failed to load orb sprite");

    let mut state = GameState::new(
        config,
        player_sprite,
        enemy_small_sprite,
        enemy_medium_sprite,
        enemy_heavy_sprite,
        enemy_large_sprite,
        enemy_elite_sprite,
        boundary_shield_sprite,
        orb_sprite,
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
