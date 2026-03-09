# LCDshootsystem — TASKS (Agentic Implementation Plan)

This plan assumes the current repo layout:
- `src/` — `main.rs`, `config.rs`, `save.rs`, `upgrade_catalog.rs`, `game/` (mod.rs, game_buff.rs, game_combat.rs, game_draw.rs, game_orb.rs, game_shop.rs, game_spawn.rs), plus `boundary.rs`, `debug_log.rs`, `drone.rs`, `enemy.rs`, `input.rs`, `orb.rs`, `player.rs`, `projectile.rs`, `render.rs`, `shield.rs`, `sprite.rs`, `text.rs`, `upgrade.rs`
- `assets/` — sprites, fonts
- `config.toml` — runtime tuning overrides (serde + TOML)
- Rust + Cargo, native dev builds, WASM for release

If anything conflicts:
- Gameplay rules: `SPEC.md`
- Tooling/process: `CLAUDE.md`
- This file sets the implementation order.

---

## Phase 1 — MVP (Core loop only; no menus, no meta, **no tests required**)

### P1.0–P1.7 ✓ DONE
Scaffolding, config, rendering (320×180), input (1D), player, lane visuals, enemies (Small/Medium/Heavy/Large), shields & death, boundary breach system.

### P1.8 Upgrade orbs (two-phase) ✓ DONE (needs gameplay verification)
Two-phase orbs: shoot to activate, collect to gain upgrade. Spawning, movement, collision all implemented.

### P1.9 Upgrade tracks ✓ DONE
All OrbTypes implemented: Shield, Damage, FireRate, Burst, Pierce, Stagger, Drone, Explosive. Offense orbs are temporary refreshable buffs. Pool gating by active buffs.

### P1.9b Explosive Shield ⚠ PARTIAL
Detonation logic done. **Remaining**: visual/audio polish; gameplay verification.

### P1.10 Drone system ✓ DONE
Attached drones persist for run, auto-fire, interact with orbs. Drone orb in normal pool.

### P1.11 Time-based scaling ⚠ PARTIAL
HP/speed/shield scaling working. Debug biome start fixed. **Remaining**: verify Medium/Heavy/Large intro times; verify orb spawn interval ramp.

### P1.14–P1.16 ✓ DONE
MVP polish (explosion FX, title/pause screens, HUD), UI polish (animation, fonts, logo), screen flash on shield loss.

### P1.17–P1.18 ✓ DONE
Biome progression (4-biome loop with boss blocking), biome-gated orb/upgrade pool.

### P1.19–P1.20 ✓ DONE
Event placeholders + HUD polish; XL enemy in DeepSpace spawn pool; boss placeholder fires at end of every biome.

**Phase 1 DoD:** All met except touch input (broken — see P2.0) and gameplay verification (P1.8, P1.9b, P1.11).

---

## Phase 2 — UX + summaries + tests allowed

### P2.0 Touch controls rework ⚠ NEEDS DESIGN
- Current touch input implementation is broken / not fit for use.
- Intended model (per SPEC §15): vertical touch strip on left side of screen; drag up/down maps to movement axis; release → 0.
- Requires testing in a mobile browser (WASM build).
- All menus/settings must be touch-operable once complete.
- Do not advertise touch controls in any in-game UI until this is done.

### P2.1 Start screen
- Add a start screen state with "Start Run" button (touch/click/key).
- Minimal styled screen consistent with game aesthetic.

### P2.2 Game over + summary
- On death: transition to summary screen.
- Show: time survived, enemies killed, upgrades collected.
- "Run Again" and "Back to Title" buttons (touch-operable).

### P2.3 Settings & configuration screen
- Touch-first settings screen accessible from title screen.
- Rotate Input toggle.
- Expose key tuning values for in-game adjustment (SPEC §17):
  - MVP set: player speed, starting shields, enemy spawn rate modifier, fire rate.
  - Expand in later phases.
- Changes made in the settings screen are **written back to the runtime config file** so they persist across sessions.
- For WASM builds: use browser local storage as config file equivalent (serde to/from JSON string).
- Any other minimal settings (e.g., audio volume placeholder).

### P2.4 WASM build & deploy pipeline
- Document `cargo build --target wasm32-unknown-unknown --release` workflow.
- Provide `index.html` shell for loading the WASM module.
- Verify touch controls work in mobile browsers.

### P2.5 Virtual slot system (boundary visual variety)
- Conceptually divide the enemy lane into 6 vertical slots.
- Assign each enemy kind a slot span: Small=1, Medium/Heavy=2, Large/XL=3.
- When an enemy transitions to Breaching, snap its Y to the nearest unoccupied slot center (with small jitter to avoid grid look).
- Track occupied slots in `BoundaryController`; release on breach resolution or stagger.
- No gameplay effect — purely cosmetic, preventing visible column queues.
- Deferred from P1.7 breach system implementation.

### P2.6 Tests (optional but allowed in Phase 2)
- Add integration/unit tests for:
  - Orb activation: shots reduce HP → orb activates → player collects.
  - Breach lock: only one enemy breaches at a time; queued enemies wait.
  - Elite pauses enemy spawns but not orb spawns.
  - Input mapping toggle correctness.
- Use `#[cfg(test)]` modules and `cargo test`.

---

## Phase 3 — Saves + meta progression

### P3.0 Persistence foundation ✓ DONE (2026-03-08)
`src/save.rs`: SaveData with RunRecord/OrbStats/PermanentUpgrades/StoryProgress. Native=JSON file, WASM=quad-storage. Save on death.

### P3.1 Permanent upgrade shop ✓ DONE (2026-03-08)
`upgrades.toml` catalog (6 upgrades), `src/upgrade_catalog.rs`, `src/game/game_shop.rs`. Title screen PLAY/UPGRADES menu. `apply_permanent_upgrades()` on run start.

### P3.2 Run summary & high score UI ⚠ NOT STARTED
Depends on: P2.2 game-over screen, P3.0 persistence.
- On game-over screen: show "NEW BEST" callout if `run_time` / `kills` / `loop_count` beat `save.best_*`.
- Display lifetime stats (total runs, lifetime kills) on title screen or a new stats screen.
- Run history accessible from title screen (last N runs, scrollable list).

### P3.3 Meta points economy ⚠ NOT STARTED
Depends on: P3.0, P3.1.
- Define meta point earning formula (e.g. kills + biomes × multiplier, shown on game-over screen).
- Persist `meta_points` / `meta_points_lifetime` in `save.rs` (fields already present).
- Optional difficulty modifiers (harder = better meta point multiplier).

### P3.4 Save slots ⚠ NOT STARTED
Depends on: P3.3.
- Multiple named player profiles (save slots).
- Each slot is a separate save file (native: `lastconvoy_save_<slot>.json`; WASM: separate localStorage keys).
- Save slot selection screen from title; delete slot option.

---

## Phase 4 — Story layer

### P4.0 Story beat system ⚠ NOT STARTED
Depends on: P3.0 persistence.
- Static pixel/comic panel renderer: draw full-screen or letterboxed panel sequence, advance on input.
- `StoryBeat` struct: id (string key), panel asset path(s), dialogue lines, trigger condition.
- Trigger conditions: first run, first time reaching a biome, Nth run, etc.
- On trigger: record beat id in `save.story_progress.seen_beats`; never replay seen beats.
- Beat assets: `art/story/base.aseprite` already started; export panels to `assets/story/`.

### P4.1 Character unlocks ⚠ NOT STARTED
Depends on: P4.0.
- Characters: Custodian (player), Rael, Voss, The Entity (per STORY.md).
- Unlock conditions tied to story beat completion or meta progression milestones.
- Persist unlocked character ids in `save.story_progress.unlocked_characters` (field already present).
- Character selection screen on title; each character may modify starting loadout or passive.

### P4.2 Story quest runs ⚠ NOT STARTED
Depends on: P4.1.
- Gated run modes unlocked by story progress (different starting conditions, special enemy patterns, or narrative overlays mid-run).
- Story quest completion recorded in `save.story_progress.seen_beats` as milestone keys.
