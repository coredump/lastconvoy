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
use sprite::{AnimMode, SpriteSheet};

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

    let player_tex = load_texture("assets/sprites/player/player_sprite_sheet.png")
        .await
        .expect("Failed to load player sprite sheet");
    let player_sprite = SpriteSheet::new(player_tex, 24.0, 16.0, 5, 0.1, AnimMode::Forward);

    let enemy_small_tex = load_texture("assets/sprites/enemies/enemy_small_sprite_sheet.png")
        .await
        .expect("Failed to load enemy_small sprite sheet");
    let enemy_medium_tex = load_texture("assets/sprites/enemies/enemy_medium_sprite_sheet.png")
        .await
        .expect("Failed to load enemy_medium sprite sheet");
    let enemy_heavy_tex = load_texture("assets/sprites/enemies/enemy_heavy_sprite_sheet.png")
        .await
        .expect("Failed to load enemy_heavy sprite sheet");
    let enemy_large_tex = load_texture("assets/sprites/enemies/enemy_large_sprite_sheet.png")
        .await
        .expect("Failed to load enemy_large sprite sheet");

    let enemy_small_sprite =
        SpriteSheet::new(enemy_small_tex, 16.0, 16.0, 5, 0.2, AnimMode::PingPong);
    let enemy_medium_sprite =
        SpriteSheet::new(enemy_medium_tex, 24.0, 24.0, 6, 0.35, AnimMode::Forward);
    let enemy_heavy_sprite =
        SpriteSheet::new(enemy_heavy_tex, 32.0, 24.0, 9, 0.1, AnimMode::Forward);
    let enemy_large_sprite =
        SpriteSheet::new(enemy_large_tex, 40.0, 32.0, 6, 0.2, AnimMode::PingPong);

    let mut state = GameState::new(
        config,
        player_sprite,
        enemy_small_sprite,
        enemy_medium_sprite,
        enemy_heavy_sprite,
        enemy_large_sprite,
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
