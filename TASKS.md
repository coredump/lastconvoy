# LCDshootsystem — TASKS (Agentic Implementation Plan)

This plan assumes the current repo layout:
- `src/` — `main.rs`, `config.rs`, `game/` (mod.rs, game_buff.rs, game_combat.rs, game_draw.rs, game_orb.rs, game_spawn.rs), plus `boundary.rs`, `debug_log.rs`, `drone.rs`, `elite.rs`, `enemy.rs`, `input.rs`, `orb.rs`, `player.rs`, `projectile.rs`, `render.rs`, `shield.rs`, `sprite.rs`, `text.rs`, `upgrade.rs`
- `assets/` — sprites, fonts
- `config.toml` — runtime tuning overrides (serde + TOML)
- Rust + Cargo, native dev builds, WASM for release

If anything conflicts:
- Gameplay rules: `SPEC.md`
- Tooling/process: `CLAUDE.md`
- This file sets the implementation order.

---

## Phase 1 — MVP (Core loop only; no menus, no meta, **no tests required**)

### P1.0 Project scaffolding & config ✓ DONE
Module structure under `src/` (see preamble), config.rs compile-time defaults, config.toml runtime overrides (serde + TOML). All tuning constants centralised; silent fallback if config.toml missing or malformed.

### P1.1 Rendering & scaling ✓ DONE
320×180 RenderTarget, integer-scaled blit to screen, letterbox, landscape-always.

### P1.2 Input plumbing (1D) ✓ DONE
Keyboard (W/S/Up/Down), gamepad (left stick Y + dpad), touch (left-strip drag); single f32 axis out. Rotate Input mode flag in config.

### P1.3 Player ✓ DONE
Fixed X, vertical movement, auto-fire projectiles, clamp to playfield.

### P1.4 Lane visuals ✓ DONE
Four fixed vertical bands rendered with palette colours; energy-rail dividers; structural border framing.

### P1.5 Enemies ✓ DONE
EnemyKind enum (Small/Medium/Heavy/Large), time-gated introduction, shielded variant ramp, movement + collision, boundary arrival + breach wind-up.

### P1.6 Shields & death ✓ DONE
N shield segments; 1 damage = 1 segment; 0 segments + damage = death → immediate restart. Vec<ShieldSegment> / ShieldSystem in shield.rs used throughout. Shield loss has visual feedback.

### P1.7 Boundary breach system ✓ DONE
Breach-lock + wind-up + simultaneous-breach window (0.10 s) + re-breach cooldown. Enemies stack behind slower/stopped ones (no overlap). Old slot-based model removed. Stagger releases lock early; explosive detonation clears breach group + micro-stall.

### P1.8 Upgrade orbs (two-phase) ✓ STRUCTURALLY COMPLETE
- Orb entity: position, HP, activated flag, current upgrade type, kind.
- Continuous timed spawning in upgrade lane (rate increases with time, from config).
- Max active orb cap enforced; if at cap, delay next spawn.
- Orbs move right → left in upgrade lane.
- **Phase 1 — Activation:**
  - Orb spawns with HP > 0, not yet activated.
  - All shots (player + drone) reduce orb HP.
  - At HP = 0 → orb becomes activated (visual change). Type is fixed at spawn.
- **Collection:**
  - Player hitbox overlaps activated orb → collect, apply upgrade.
- Orbs that exit left edge despawn.
- Orb spawning continues during elite events.
- **Implementation status**: Orb struct with OrbPhase::Inactive/Active; take_hit() logic correct; spawning, movement, collision, Active-only collection all implemented. Needs gameplay verification.

### P1.9 Upgrade tracks ✓ UPDATED (offense converted to temporary buffs)
Implemented OrbTypes: Shield, Damage, FireRate, Burst, Pierce, Stagger, Drone.
- **Shield**: +1 shield per collection (up to cap 3). Skipped from pool when full. ✓
- **Damage buff**: temporary flat damage boost while active (refresh on re-collect; no tier stacking). ✓
- **FireRate buff**: temporary shot-interval reduction while active (refresh on re-collect; no tier stacking). ✓
- **Burst buff**: temporary periodic burst-shot readiness while active (refresh on re-collect; no tier stacking). ✓
- **Pierce buff**: temporary extra pierce while active; same-enemy double-hit bug remains fixed. ✓
- **Stagger buff**: temporary knockback on hit for Small/Medium/Heavy while active. ✓
- **Drone**: in normal orb spawn pool; attached drone implemented and active. ✓
- **Explosive Shield**: core behavior implemented; remaining polish/verification tracked in P1.9b. ⚠
- **Pool gating**: active offense buff types are excluded from orb spawn pool until expiry; Shield/Explosive/Drone gates unchanged. ✓

### P1.9b Explosive Shield ⚠ PARTIAL
- Detonation logic implemented: `trigger_explosive_shield()` kills non-elite enemies in zone, pushes Large/Elite back, clears breach group, applies micro-stall. ✓
- `ShieldSystem::convert_to_explosive()` implemented. ✓
- `OrbType::Explosive` collection wired to `convert_to_explosive()`. ✓
- **Remaining**: visual/audio for explosion (flash, particle hints); gameplay verification that detonation + stall reads correctly in play.

### P1.10 Drone system ✓ DONE
Attached drones persist for the run, positioned relative to player, auto-fire on same timer. Drone shots interact with orbs equally to player shots. Drone orb in normal pool.

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
- Elite is a DPS check: if not killed before boundary, enters breach wind-up and deals damage like any other enemy.
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
- Boundary behavior: enters breach wind-up and deals damage via the standard breach mechanism.
- On death: resume spawning, apply small scaling bump.
- **Status**: miniboss_timer exists in GameState but no spawn or event logic implemented.

### P1.14 MVP polish ✓ COMPLETE
- Orb activation state change (color/glow shift). ✓ Color tint changes per phase in draw_orbs()
- Enemy destruction (small particle burst or flash). ✓ Implemented (explosion_2 sprite, 5 frames 40ms each)
- Object pooling for projectiles and enemies. ✓ Using Vec<T> with retain()
- Frame-rate independence: all movement/timers use `get_frame_time()` delta. ✓ Implemented throughout
- Title & pause screens. ✓ Implemented (at_title state, any-key-to-start; paused state, P+ESC toggle; controls overlay)
- Upgrade HUD redesign. ✓ Drone placeholder always visible; Shield/Explosive removed from HUD; expiring buffs shown with vertical timer bars only when active

### P1.15 UI polish ✓ DONE (2026-03-02)
Per-frame animation durations in sprite.rs, monogram_font + logo_sprite, title screen logo, floating text font switch, run timer centering.

**Phase 1 DoD (Definition of Done)**
- Playable loop: start → title screen → any-key-to-start → survive → die → restart. ✓ WORKING
- Pause overlay (P+ESC) with controls list implemented. ✓ WORKING
- Explosion effect on enemy death visible. ✓ WORKING
- Upgrade HUD with drone placeholder and vertical timer bars. ✓ COMPLETE
- Touch input works (at least in WASM build). ⚠ **BROKEN — see P2.0**
- Orbs work exactly as specified (activate then collect). ✓ STRUCTURALLY COMPLETE (needs gameplay verification)
- Elites and Mini-Bosses work with enemy spawn pause (orbs continue). ⚠ NOT STARTED (P1.12/P1.13 blocking)
- Boundary breach lock and compression work. ✓ COMPLETE
- No menus required. ✓ (title/pause screens are state overlays, not full menus)
- No tests required. ✓
- Runs natively for dev; builds to WASM for browser. ✓ WASM build exists

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
- Assign each enemy kind a slot span: Small=1, Medium/Heavy=2, Large/Elite=3.
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
- Save slots (serde + local storage for WASM, filesystem for native).
- Meta points economy.
- Permanent upgrades (starting shields/drones, unlock types, increase drops).
- Optional difficulty modifiers for better rewards.

---

## Phase 4 — Story layer
- Static pixel/comic panels + dialogue.
- Character unlocks and gated "story quest" runs.
