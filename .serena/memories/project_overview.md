# lastconvoy — Project Overview

## Purpose
Browser-based minimalist systemic roguelite shooter ("LCDShootSystem").
Core feel: lane pressure / tower-defense, not bullet-hell dodging.
Continuous run, ends only on death. Typical run: 12–18 minutes.

## Tech Stack
- **Language:** Rust (stable, edition 2024)
- **Build tool:** Cargo
- **Key crates:** macroquad ~0.4 (rendering/input/window), serde 1 + toml 0.8 (config)
- **Dev target:** native (`cargo run`)
- **Release target:** WASM (`cargo build --target wasm32-unknown-unknown --release`)

## Source Structure (`src/`)
| File | Concern |
|------|---------|
| `main.rs` | Entry point, window conf, main loop |
| `config.rs` | Compile-time consts + RuntimeConfig (Option<T>) + resolved Config + load/save |
| `game.rs` | GameState — owns all entities, update/draw |
| `player.rs` | Player struct |
| `enemy.rs` | EnemyKind enum + Enemy struct |
| `projectile.rs` | ProjectileSource enum + Projectile struct |
| `orb.rs` | OrbType, OrbPhase enums + Orb struct |
| `drone.rs` | Drone struct |
| `shield.rs` | ShieldSegment struct |
| `upgrade.rs` | UpgradeTrack enum |
| `elite.rs` | EliteVariant enum + EliteEvent struct |
| `boundary.rs` | Boundary with slot tracking |
| `input.rs` | InputState { axis: f32 } |
| `render.rs` | RenderPipeline stub (filled in P1.1) |

## Architecture Rules
- No ECS. Plain structs + Vec<T> + GameState.
- Frame-rate independent: all movement/timers use `get_frame_time()`.
- Internal resolution: **320×180** always. Integer scaling only (×2–×6). Letterbox.
- Render pipeline: draw to 320×180 RenderTarget, blit to screen.
- Entity pools: Vec<T> with retain() or swap-remove.
- Config: compile-time defaults in config.rs; runtime overrides via config.toml (TOML format).
- `#![allow(dead_code)]` at crate root during Phase 1 (stubs are intentionally incomplete).

## Gameplay Constraints (non-negotiable)
- Internal res: 320×180, landscape, integer scaling only.
- Single-axis player movement (vertical).
- Two lanes: enemies top (rows 16–119), upgrades bottom (rows 124–163).
- Time-based scaling only. No rubber-banding.
- Orbs: two-phase (activate THEN cycle). Activation hits do NOT cycle.
- Drone shots NEVER interact with orbs.
- Boundary: finite slots; when full, lane jams — no overlap/pushing.
- EnemyElite1 and MiniBoss: event-only, never in regular spawn pool.

## Naming Conventions
Player, Enemy, EnemyKind, Orb, Drone, ShieldSegment, EliteEvent
Commit messages: `P1.X: brief description`
One concern per module. Keep main.rs thin. No `unsafe` unless unavoidable.

## Current Phase
Phase 1 MVP. P1.0–P1.7 COMPLETE. P1.8 (orbs two-phase) STRUCTURALLY COMPLETE. P1.9–P1.14 partially implemented. Next: Verify P1.8, fix P1.6 shields model, implement P1.9+ upgrades and events. No tests required until Phase 2.
