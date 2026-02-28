# LCDshootsystem — TASKS (Agentic Implementation Plan)

This plan assumes the current repo layout:
- `src/main.rs` (hello-world entry point)
- `Cargo.toml` with `macroquad` dependency
- `assets/` (to be created)
- Rust + Cargo, native dev builds, WASM for release

If anything conflicts:
- Gameplay rules: `SPEC.md`
- Tooling/process: `CLAUDE.md`
- This file sets the implementation order.

---

## Phase 1 — MVP (Core loop only; no menus, no meta, **no tests required**)

### P1.0 Project scaffolding & config ✓ DONE
- Create module structure under `src/`:
  - `config.rs` — all tuning constants (`pub const` or a `Config` struct)
  - `game.rs` — top-level game state struct and main update/draw loop
  - `input.rs` — 1D input system
  - `player.rs` — player entity
  - `enemy.rs` — enemy types and spawning
  - `projectile.rs` — projectile entity and pool
  - `orb.rs` — upgrade orb entity
  - `drone.rs` — drone entity
  - `shield.rs` — shield segment model
  - `upgrade.rs` — upgrade track definitions
  - `elite.rs` — elite event system
  - `boundary.rs` — boundary slot and jam system
  - `render.rs` — integer-scaled render-to-texture pipeline
- Centralize all tuning constants in `config.rs` as compile-time defaults:
  - lane bounds, speeds, caps, timers, scaling curves
  - boundary slot count
  - orb caps, orb HP curve
  - elite interval + random offset range
- Implement **runtime config file** loading (see SPEC §17):
  - Use TOML format with serde crate dependency.
  - Define a serializable `RuntimeConfig` struct covering key tuning values.
  - On startup: attempt to load from `config.toml` next to the binary.
  - If missing or malformed: fall back to compile-time defaults, log a warning, continue.
  - Write a default config file on first run if none exists (so users have a template to edit).
- Set up `main.rs` to create a macroquad window, load config, instantiate game state, run the loop.
- Create `assets/` directory for sprites (can start with placeholder rects).

### P1.1 Rendering & scaling ✓ DONE
- Create an offscreen `RenderTarget` at 320×180 (internal resolution).
- Each frame: draw all gameplay to the render target, then blit to screen with integer scaling.
- Compute largest integer scale that fits the window (×2 through ×6).
- Letterbox (black bars) when window aspect doesn't match; center the scaled image.
- Portrait windows: gameplay stays landscape, letterbox top/bottom — never rotate or pause.

### P1.2 Input plumbing (1D) ✓ DONE
- Read keyboard state each frame:
  - Default: W/S + Up/Down → produce axis value -1/0/+1.
  - Rotate Input mode (config flag): A/D + Left/Right instead.
  - Opposing keys cancel to 0.
- Read gamepad if connected:
  - Default: left stick Y + dpad up/down.
  - Rotate Input mode: left stick X + dpad left/right.
  - Apply analog deadzone from config.
- Touch input (for WASM):
  - Detect vertical drag on left 20% of screen.
  - Map drag delta to axis value; release → 0.
- Combine sources: keyboard OR gamepad OR touch (last-active wins or max magnitude).
- Output: single `f32` axis value in [-1.0, 1.0].

### P1.3 Player ✓ DONE
- Player entity: position fixed at left side (x from config), y moves vertically.
- Clamp y within playfield bounds (top border bottom edge to bottom border top edge).
- Apply input axis × speed × delta_time to y each frame.
- Auto-fire: spawn a projectile forward on a timer (fire rate from config).
- Projectile travels right at configured speed; despawns off-screen.
- Player projectiles tagged as `source: Player` (matters for orb interaction).

### P1.4 Lane visuals ✓ DONE
- Draw top border (rows 0–15), enemy lane background (16–119), divider (120–123), upgrade lane background (124–163), bottom border (164–179).
- Divider: 4 px, styled as energy rail / barrier (not flat color).
- Borders: styled as structural framing (not UI bars).
- Use placeholder colors from the palette spec; refine art later.

### P1.5 Enemies (small / medium / heavy / large) ✓ DONE (spawning + movement + collision + boundary arrival + shielded enemies + stacking)
- Define an `EnemyKind` enum: `Small`, `Medium`, `Heavy`, `Large`.
- Each kind has: size, HP, speed, boundary behavior, sprite (placeholder rect initially).
- Continuous spawning system driven by a timer (interval from config, decreases over time).
  - Early run (~first 60s): mostly Small.
  - Medium introduced after config threshold time.
  - Heavy introduced later than Medium.
  - Large introduced later than Heavy.
- Movement: right → left within enemy lane bounds.
- Collision with player projectiles: reduce HP, despawn projectile (unless piercing).
- Boundary arrival behavior:
  - Small: trigger 1 damage event on player, then despawn.
  - Medium / Heavy / Large: stop at boundary, occupy a boundary slot, tick damage at interval until destroyed.
- Shielded enemies:
  - Some enemies spawn with an extra shield HP layer (must be broken before main HP).
  - No shield regen.
  - Frequency increases slowly over time (config curve).

### P1.6 Shields & death ✓ DONE
- Player has N shield segments (starting count from config).
- Each damage event removes exactly 1 segment.
- 0 segments + damage event → player death.
- Shield loss: visual feedback (flash, segment disappears).
- On death: immediately reset game state and restart (no game-over screen yet).
- Losing shields never removes drones or reduces offense.
- **NOTE**: Vec<ShieldSegment> conversion complete. ShieldSystem in shield.rs used throughout.

### P1.7 Boundary slots & jam ✓ DONE
- Define a fixed number of boundary slots (from config).
- When a Medium/Heavy/Large/Elite reaches the left boundary:
  - If a slot is free: occupy it, begin damage ticking.
  - If no slot is free: enemy stops in place (queued), does NOT tick damage.
- When a slotted enemy is destroyed: free its slot; nearest queued enemy (if any) advances to fill it.
- No overlap, no pushing physics.

### P1.8 Upgrade orbs (two-phase) ✓ STRUCTURALLY COMPLETE
- Orb entity: position, HP, activated flag, current upgrade type, kind.
- Continuous timed spawning in upgrade lane (rate increases with time, from config).
- Max active orb cap enforced; if at cap, delay next spawn.
- Orbs move right → left in upgrade lane.
- **Phase 1 — Activation:**
  - Orb spawns with HP > 0, not yet activated.
  - Only projectiles with `source: Player` (not drones) reduce orb HP.
  - Hits during activation do NOT cycle type.
  - At HP = 0 → orb becomes activated (visual change).
- **Phase 2 — Cycling:**
  - Only activated orbs can be cycled.
  - Only `source: Player` projectiles cycle type (one step per hit).
  - Drone projectiles never interact with orbs.
- **Collection:**
  - Player hitbox overlaps orb → collect, apply selected upgrade.
- Orbs that exit left edge despawn.
- Orb spawning continues during elite events.
- **Implementation status**: Orb struct with OrbPhase::Inactive/Active; take_hit() logic correct; spawning, movement, collision, Active-only collection all implemented. Needs gameplay verification.

### P1.9 Upgrade tracks ✓ MOSTLY DONE (Drone effect + Explosive Shield pending)
Implemented OrbTypes: Defense, Damage, FireRate, Burst, Pierce, Stagger, Drone.
- **Defense**: +1 shield per collection (up to cap 3). Skipped from pool when full. ✓
- **Damage** (3 levels): flat damage per shot scales with level. ✓
- **FireRate** (3 levels): shot interval decreases per level. ✓
- **Burst** (3 levels): periodic double-damage shot on separate cooldown. ✓
- **Pierce** (3 levels): shot passes through N additional enemies; same-enemy double-hit bug fixed. ✓
- **Stagger** (1 level): on hit, knocks back Small/Medium/Heavy (including slotted enemies). ✓
- **Drone**: collection wired; drone firing not yet implemented (see P1.10). ⚠
- **Explosive Shield**: not yet implemented (see P1.9b below). ⚠

### P1.9b Explosive Shield ⚠ NOT STARTED
- Converts one normal shield segment to an explosive segment (max one at a time).
- Explosive segment breaks last (after all normal segments).
- On break: explosion 40 px forward from boundary, full enemy lane height.
  - Elite and Mini-Boss pushed back 24 px (unless occupying a boundary slot).
  - Non-elite enemies not pushed back.
  - No effect on upgrade lane.
- Upgrade unavailable if player has 0 segments or an explosive segment already exists.
- Requires separate OrbType or a modifier flag on the Defense orb (design TBD).

### P1.10 Drone system ⚠ STUB ONLY
- Attached drones: persist for the run, positioned relative to player.
- Each drone auto-fires on same timer as player.
- Drone projectiles tagged `source: Drone` — they damage enemies but do NOT interact with orbs.
- (Detached / cross-lane drones deferred to later upgrade pool expansion.)
- **Status**: Drone struct exists with x/y/fire_timer/attached/ttl. GameState has drones: Vec<Drone> but drones are never updated, fired, or positioned in game loop.

### P1.11 Time-based scaling baseline ⚠ PARTIAL
- All scaling curves defined in `config.rs`.
- Time-only ramps (no kill-count or player-power triggers):
  - Enemy spawn interval decreases (more enemies over time). ✓ IMPLEMENTED
  - Medium / Heavy / Large introduction times. ⚠ NEEDS VERIFICATION
  - Enemy HP slow ramp for Medium+ tiers. ⚠ NEEDS VERIFICATION
  - Shielded enemy frequency ramp. ✓ IMPLEMENTED (enemy.rs spawn_enemy checks SHIELDED_FREQ_SCALE * run_time)
  - Orb spawn interval decreases (more orbs over time). ⚠ NEEDS VERIFICATION
- Tuneable: keep readability-first, avoid bullet-sponge slog or exponential blowup.

### P1.12 Elite events ⚠ NOT STARTED
- Elite event system with its own timer (interval + random offset, from config).
- On elite trigger:
  - Pause normal enemy spawning.
  - Randomly choose variant A (single elite) or C (elite + support enemies).
  - Spawn `EnemyElite1` (48×40 px, from config HP) in enemy lane, moving right → left.
  - Orb spawning continues uninterrupted.
- Elite is a DPS check: if not killed before boundary, occupies a slot and ticks damage.
- On elite death:
  - Resume normal enemy spawning.
  - Apply a small global scaling bump (config-defined).
- `EnemyElite1` is never added to the regular continuous spawn pool.
- **Status**: EliteEvent struct exists with active/variant/timer fields. Elite spawning can happen via pick_enemy_kind (debug mode only). No timer countdown, no pause/resume logic implemented.

### P1.13 Mini-Boss events ⚠ NOT STARTED
- Separate timer from elites (interval + random offset, from config).
- On trigger:
  - Pause normal enemy spawning.
  - Spawn one Mini-Boss (64×48 px, 25–40 HP scaled by time) in enemy lane.
  - Orb spawning continues.
- Boundary behavior: occupy slot + tick damage (like Large/Heavy).
- On death: resume spawning, apply small scaling bump.
- **Status**: miniboss_timer exists in GameState but no spawn or event logic implemented.

### P1.14 MVP polish (still Phase 1) ⚠ PARTIAL
- Visual feedback for:
  - Shield loss (segment flash/pop). ⚠ Needs Vec<ShieldSegment> implementation
  - Orb activation state change (color/glow shift). ✓ Color tint changes per phase in draw_orbs()
  - Elite/Mini-Boss arrival (subtle screen cue, no stage screen). ⚠ Not yet (no events)
  - Enemy destruction (small particle burst or flash). ⚠ Not visible
- Object pooling for projectiles and enemies if needed for performance. ✓ Using Vec<T> with retain()
- Frame-rate independence: all movement/timers use `get_frame_time()` delta. ✓ Implemented throughout

**Phase 1 DoD (Definition of Done)**
- Playable loop: start instantly → survive → die → restart. ✓ WORKING
- Touch input works (at least in WASM build). ⚠ Touch stub only
- Orbs work exactly as specified (activate then cycle then collect). ✓ STRUCTURALLY COMPLETE (needs gameplay verification)
- Elites and Mini-Bosses work with enemy spawn pause (orbs continue). ⚠ NOT STARTED (no event pause logic)
- Boundary slots and jam work. ✓ COMPLETE
- No menus required. ✓
- No tests required. ✓
- Runs natively for dev; builds to WASM for browser. ✓ WASM build exists

---

## Phase 2 — UX + summaries + tests allowed

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

### P2.5 Tests (optional but allowed in Phase 2)
- Add integration/unit tests for:
  - Orb activation vs cycling (activation hits do not cycle).
  - Drone shots never affect orbs.
  - Boundary slot cap and jam behavior.
  - Elite pauses enemy spawns but not orb spawns.
  - Input mapping toggle correctness.
- Use `#[cfg(test)]` modules and `cargo test`.

---

## Phase 3 — Saves + meta progression
- Save slots (serde + local storage for WASM, filesystem for native).
- Meta points economy.
- Permanent upgrades (starting shields/drones, unlock types, increase drops).
- Optional difficulty modifiers for better rewards.

---

## Phase 4 — Story layer
- Static pixel/comic panels + dialogue.
- Character unlocks and gated "story quest" runs.
