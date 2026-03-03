# LCDshootsystem — SPEC (Gameplay Source of Truth)

If anything in the repo conflicts with this document regarding gameplay rules, **this SPEC wins**.

## 0. Document hierarchy
- **Gameplay truth:** `SPEC.md` (this file)
- **Implementation order:** `TASKS.md`
- **Tooling/process + repo conventions:** `CLAUDE.md`
- If conflict: gameplay → SPEC, process/tooling → CLAUDE, order → TASKS.

## 1. Identity & goals
- Browser-based minimalist systemic roguelite shooter.
- Core feel: **lane pressure / tower-defense**, not bullet-hell dodging.
- Continuous run, **ends only on death** (no stages, no "final boss" end condition).
- Typical successful run: ~12–18 minutes. Long outliers can happen but are not a target.
- Structured pixel-art aesthetic. Clean silhouettes, strict palette control. Neon reserved for shots and interactive elements.

## 2. Orientation & rendering
- Canonical world orientation: **landscape**.
- World/camera never rotates.
- If viewport is portrait on WASM: apply CSS `rotate(90deg)` to the canvas element so the landscape game fills the screen. Game world, UI, and movement axes are unchanged. Scale uses swapped screen dimensions. On native, letterbox as normal.
- Scaling: Integer only (×1, ×2, ×3, ×4, ×5, ×6). No fractional scaling.

## 3. Lanes
Three gameplay lanes (always):
- **Top upgrade lane:** Upgrades (orbs)
- **Enemy lane:** Enemies
- **Bottom upgrade lane:** Upgrades (orbs)
Each upgrade lane is visually smaller than the enemy lane.

Lanes are independent movement/combat spaces, but orb spawning is synchronized across both upgrade lanes (see §9).

Coordinate ranges (locked):

| Section       | Rows     | Height |
|---------------|----------|--------|
| Top Border    | 0–20     | 21 px  |
| Top Upgrade Lane | 21–42 | 22 px  |
| Enemy Lane    | 43–157   | 115 px |
| Bottom Upgrade Lane | 158–179 | 22 px |

## 4. Player
- Fixed on left side.
- Single-axis movement (vertical only) within playfield (and between lanes).
- Auto-fires forward.
- Shoots only the lane the ship is in (cross-lane influence only via **temporary remote drones**, see §7).

Internal movement axis model:
- -1 = up, 0 = neutral, +1 = down

Canonical size: 24×16 px recommended, 32×16 px maximum.
Sprite padding: 2 px minimum inside bounding box.

## 5. Enemies
Enemies move right → left in enemy lane. Enemies do not shoot.

Canonical enemy size table:

| Tier      | Size (W×H) | Bounding box |
|-----------|------------|--------------|
| Small     | 16×16      | 14×14 opaque content, 1 px transparent border |
| Medium    | 24×24      | 2 px padding |
| Heavy     | 32×24      | 2 px padding |
| Large     | 40×32      | 2 px padding |
| Elite     | 48×40      | EnemyElite1; 2–3 px padding |
| Mini-Boss | 64×48      | event-based only; 2–3 px padding |
| Boss      | 72×72 max  | reserved; no gameplay rules yet |

Enemy lane height 115 px; max standard enemy height 48 px; boss hard cap 72 px.

Enemy classes (breach behavior for all classes: see §8):
- **Small:** 1 HP.
- **Medium:** multi HP. Introduced before Heavy.
- **Heavy:** ~6–8 HP. Introduced later than Medium.
- **Large:** high HP.
- **Mini-Boss:** ~25–40 HP. Event-based only (see §13).
- **Boss:** Reserved tier. No gameplay rules defined yet.

Wind-up times are tuning values — see `config.toml` (`windup_time_*`). A heavier enemy has a longer wind-up, giving the player more time to intercept it before the breach resolves. Enemy differentiation comes from HP, speed, and wind-up time — **not** from breach damage amount (all non-boss enemies deal exactly 1 breach event).

### Shielded enemies (additive layer)
- Some enemies spawn with a shield layer (extra HP layer that must be broken first).
- No shield regeneration.
- Shielded enemies are **additive** (do not replace normal spawns).
- Frequency increases slowly over time.

## 6. Damage, shields, death
- No instant death on contact.
- Player starts each run with **0 shield segments**.

### Shields
- Shields are multi-segment: each segment = 1 hit absorbed.
- Shield upgrade orbs grant +1 segment per collection (cap: 3).
- Each damage event removes one segment. When 0 segments remain, the next hit = death.
- HUD shows current segment count as small icons in the top-left corner.
- `player_starting_shields` in config.toml overrides the starting count (debug use).
- **No shield regen, durability, or damage reduction mechanics.**

### Explosive Shield
- The Explosive Shield upgrade converts one normal shield segment into an explosive segment.
- Only one explosive segment can exist at a time.
- The explosive segment always breaks **last** — normal segments are consumed first.
- When the explosive segment breaks: an explosion occurs 40 px forward from the barrier, spanning the full enemy lane height.
- Explosion effect on enemies:
  - Elite and Mini-Boss are pushed back 24 px.
  - Non-elite enemies within the zone are destroyed.
  - Breaching enemies in the zone are cleared; breach lock is released immediately.
  - Explosion does **not** affect the upgrade lane.
- A brief movement freeze (micro-stall, ~0.25 s) is applied after explosion for impact readability.
- The Explosive Shield upgrade allows only one explosive segment at a time. If collected with no shield segments and below cap, it grants a new explosive segment.

### Damage events (breach model)
See §8 for the full breach mechanics (lock, queuing, cooldown, stagger interactions).

Core breach resolution:
- If player has shield: remove exactly one segment (normal segments first; explosive last).
- If player has no shield: player dies instantly.
- The breaching enemy is despawned immediately after resolution.

All enemies deal exactly **1 breach event** — there is no multi-damage or spillover.

**Losing shields never removes drones and never reduces offense.**

## 7. Drones & shooting model
Definition:
- Any shooter that is not the player is a **drone**.

Drone types:
- **Attached drones:** persist for the run, move with the main ship.
- **Remote drones:** temporary; despawn after duration (TTL).

Shot-type modifiers apply to **all** shooting entities (player + drones).

Critical orb-interaction rules:
- All shots (player, attached drone, remote drone) interact with orbs equally — any shot can activate a sealed orb.
- **Remote (upgrade-lane) drones** are stationary in either upgrade lane (top or bottom). They fire rightward continuously, hitting incoming inactive orbs to accelerate activation. They despawn immediately when any orb activates.
- **Shot-only lane barrier:** an invisible 1px barrier exists on the upgrade-side edge of each enemy/upgrade boundary. Shots crossing these lines are blocked except through a left-side gate corridor.

## 8. Boundary breach & queuing
- Only one enemy may wind up at the boundary at a time (**breach lock**).
- When an enemy reaches `BOUNDARY_X` and breach is free: it enters the `Breaching` state, its movement stops, and the breach lock engages.
- If another enemy arrives within a brief simultaneous window (≤ 0.10 s) of the first, it may also join the current breach group (rare, organic simultaneous breach).
- All other enemies that reach the boundary while locked are clamped at `BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET` (24 px) and remain `Moving`. They compress naturally behind the breaching enemy, forming a visible pressure cluster.
- After all enemies in the breach group resolve, the lock is released. The frontmost compressed enemy naturally advances to `BOUNDARY_X` and starts the next breach.
- **Re-breach cooldown:** after a breach resolves naturally, a cooldown timer engages (config: `re_breach_cooldown`, default 0.4 s). During cooldown, Moving enemies at the boundary are clamped at `BOUNDARY_X`, creating a visible pressure buildup. This gives the player brief tactical breathing room but heightens the threat of the next breach. *Crucially:* if the player clears a breacher via stagger knockback or explosive shield detonation, the cooldown does NOT trigger — the next enemy advances immediately, maintaining aggressive pacing.
- Enemies that are staggered (knocked back) while Breaching are returned to Moving and removed from the breach group, releasing the lock early (and bypassing the re-breach cooldown).
- **Future (P2.X):** virtual slot system — 6 vertical lanes with span-based occupancy per enemy kind (Small=1, Medium/Heavy=2, Large=3) for visual variety at the boundary.

Enemy stacking behavior (unchanged):
- Faster enemies stack behind slower/stopped ones (no overlapping). A faster enemy that catches up to a slower one matches its effective position and waits.

## 9. Upgrade lane & orb interaction (two-phase)
Orbs are interactive and require deliberate time/aim.

### Orb spawning
- Orbs spawn continuously on a timer, attempting one spawn in each upgrade lane on each tick.
- Spawn rate slowly increases with time (difficulty).
- Small random offset ("predictable but not to the second").
- Enforce max active orb cap (global across both upgrade lanes); if at cap, delay spawn.
- Orb type is rolled independently per lane when both lanes spawn on the same tick.
- If only one global orb slot remains, only one lane spawns that tick.
- Enemy spawns pause during elites; **orb spawns continue during elites**.

### Orb movement
- Orbs move right → left in their selected upgrade lane.

### Two-phase interaction
Orbs spawn sealed (inactive).

**Phase 1 — Activation**
- Orb has HP.
- Player must shoot orb until HP reaches 0.
- All shots interact with orbs equally.

At HP = 0, orb becomes **activated** (clear visual state). Its upgrade type is fixed at spawn.

**Collect**
- Player must physically collect the activated orb to gain its upgrade.

## 10. Upgrade tracks (in-run)

Upgrades are collected by physically touching an activated orb. Each OrbType is a discrete track.

### (A) Shield
- **Shield (+1 Shield Segment)**: adds one shield segment per collection. Skipped from the orb pool when shields are already at cap (3 segments). Up to 3 collections.
- **Explosive Shield**: implemented as a shield modifier. It converts a normal segment to explosive when possible; if no segments exist and cap allows, it grants a new explosive segment. See §6 for behavior rules.

### (B) Drones
- **Drone**: adds one attached drone. Drone fires in the same lane as the player and moves with the player.
- **DroneRemote**: on pickup, spawns two temporary drones (one per upgrade lane). They fire rightward at inactive orbs to accelerate activation (Phase 1 only) and despawn when any orb activates.

### (C) Offense — Temporary buffs
Offense orbs are temporary buffs, not permanent levels. Re-collecting an active offense buff refreshes its timer; it does not stack potency.

- **Damage Buff**: while active, increases flat projectile damage.
- **FireRate Buff**: while active, decreases player shot interval.
- **Burst Buff**: while active, periodically marks the next shot as a burst shot (double-damage multiplier).
- **Pierce Buff**: while active, projectiles pass through additional enemies. A shot cannot hit the same enemy more than once.
- **Stagger Buff**: while active, hits displace Small/Medium/Heavy enemies rightward up to 12 px. Does not affect Large/Elite/Mini-Boss.

Shot modifiers apply to player shots and can also apply to attached drone shots when the corresponding config toggle is enabled.

**Pool rules:**
- Offense orb types with an active buff are excluded from the spawn pool until that buff expires.
- Shield is excluded when shields are at cap.
- No projectile size scaling.

## 11. Spawning & scaling (time-based only)
- Enemy spawns are continuous (not wave-based).
- Time-based scaling only (no kill-based triggers, no player-power rubber-banding).
- Controlled growth:
  - Spawn density increases slowly
  - Medium/large durability increases slowly
  - Shielded frequency increases slowly
- Avoid bullet-sponge slog and exponential scaling.

### Early-run safety
- First minute(s) are safer:
  - Mostly small enemies
  - Early orbs, easy to activate
  - Medium/large introduced later

## 12. Elite events (DPS checks)
Purpose: punctuate runs without formal stages.

The elite entity used in both variants is **EnemyElite1** (`enemy_elite_1_sprite_sheet.png`).
EnemyElite1 is **not** a fourth regular enemy class — it only appears during elite events,
never in the continuous enemy spawn pool.

EnemyElite1 canonical bounding box: 48×40 px (2–3 px padding).

### Timing
- Time-based interval with small random offset (predictable but not exact).

### Spawn pause behavior
On elite trigger:
- Pause normal enemy spawning.
- Spawn elite event in enemy lane.
- Upgrade orbs continue spawning.

**Not yet implemented** (P1.12). Rules below are authoritative for implementation.

### Elite variants (random per event)
- **A)** Single massive elite
- **B)** Massive elite + support enemies

### Elite behavior
- Moves right → left (approach window is DPS check).
- If not killed before boundary:
  - Enters breach resolution flow at `BOUNDARY_X` (wind-up, then one breach event).
- Elite scaling is time-only.
- Can be pushed back 24 px by an explosive shield detonation unless explicitly marked stagger-immune by event logic.

After elite death:
- Resume normal enemy spawning.
- Apply a small global scaling bump consistent with controlled growth.

## 13. Mini-Boss Events

**Not yet implemented** (P1.13). Rules below are authoritative for implementation.

Purpose: rare punctuation events distinct from Elite events.

### Timing
- Separate time-based interval from elite events with a small random offset.

### Spawn behavior
- Pause normal enemy spawning during the event.
- Spawn one Mini-Boss in the enemy lane.
- Upgrade orbs continue spawning.

### Mini-Boss behavior
- Moves right → left (approach window is a DPS check).
- If not killed before boundary: enters breach resolution flow at `BOUNDARY_X`
  (wind-up, then one breach event).
- Mini-Boss HP scales with time (time-based only, no rubber-banding).
- Can be pushed back 24 px by an explosive shield detonation unless explicitly marked stagger-immune by event logic.

### After Mini-Boss death
- Resume normal enemy spawning.
- Apply a small global scaling bump consistent with controlled growth.

## 14. UI screens (title, pause, game over)
- **Title screen**: displayed on startup; shows game name and "Press any key to start" prompt; pressing any key or clicking anywhere begins a new run (enters game state with `at_title = false`).
- **Pause screen**: toggled with P key or ESC key during gameplay; displays a semi-transparent overlay with controls list and prompt to press P or ESC to resume; pause gate gates all gameplay updates (no enemies/projectiles/orbs move while paused).
- **Game over screen**: (Phase 2+) shown on player death; displays run summary (time survived, enemies killed, breaches suffered); provides buttons to start a new run or return to title.

## 15. Inputs (gameplay requirements)
- Single-axis movement only.
- Default bindings: **W/S**, Up/Down arrows, controller stick **Y**, D-pad Up/Down.
- Provide a single setting: **Rotate Input**:
  - When enabled: A/D, Left/Right arrows, stick **X**, D-pad Left/Right.
- Opposing inputs cancel; analog deadzone applies.

## 16. Touch controls & menus
- **Movement:** Touch strip on the left of the game view (20% of canvas width). Drag up/down to move player; release returns to neutral. Full deflection at 24px drag. Portrait WASM: strip is at the top of the rotated canvas; drag left/right maps to game up/down.
- **Tap to advance:** Any tap starts the game from title screen; any tap restarts from game-over screen.
- **Pause button:** Persistent pause/resume icon in the top-right of the HUD (game coords ~300..318, 0..18). Tap toggles pause.
- **Run timer:** Displayed on the left of the top border (moved to make room for the pause button).
- All menus must be fully operable via touch.

## 17. Meta progression & story (phase-gated)
- Meta progression is not part of MVP.
- When implemented (Phase 3+), meta can:
  - Increase starting stats
  - Unlock new upgrade types
  - Increase drop frequency
  - Add optional difficulty modifiers for better rewards
- Story is Phase 4+ and must not reshape core mechanics.

## 18. Tuning constants & runtime configuration
- All tuning constants and scaling curves must have **compile-time defaults** in `src/config.rs`.
- Avoid magic numbers scattered across systems.

### Runtime config file
- The game must load tuning values from an **external config file** at startup.
- The file must be human-readable and editable with a text editor — no rebuild required to change values.
- Format: TOML (human-readable, supports comments, stable parsing).
- If the file is missing or malformed, fall back to compile-time defaults silently (no crash).
- The file should cover at least: player speed, fire rate, enemy speeds/HP, spawn intervals, scaling curves, breach timings/cooldowns, orb caps/HP, elite/mini-boss intervals, shield starting count.
- Changes to the file take effect on next game launch (hot-reload is not required).

### In-game configuration screen
- The game must provide a configuration/settings screen accessible from the title screen.
- The settings screen must expose key tuning values for adjustment without editing the file.
- Any values changed in the settings screen must be written back to the config file so they persist across sessions.
- The settings screen must be fully touch-operable (consistent with §15).
- Minimal viable settings for MVP: player speed, starting shields, enemy spawn rate modifier, fire rate. Expand in later phases.

## 19. Tooling pointers (non-authoritative)
- The repo uses macroquad + Rust + Cargo.
- Native builds for development; WASM (wasm32-unknown-unknown) for browser release.
- Prefer official macroquad docs and examples:
  - https://macroquad.rs
  - https://github.com/not-fl3/macroquad

## 20. Visual Specification

### Palette (v3)
- Outline: Pure Black (#000000). Outline is NOT part of any material ramp.
- Each material uses exactly 3 shades. No 4th shade.
- Darkest interior shade must be clearly lighter than black.
- No highlight approaches white.
- No mixing ramps unless design requires it.

Materials: Space, Steel, Player Cyan, Upgrade Teal, Infection Magenta, Glass,
Decal Orange, Decal Yellow, Damage Red, Warp Cyan.

### Sprite Padding Rules
| Class       | Padding   |
|-------------|-----------|
| Projectile  | 0–1 px    |
| Small       | 1 px      |
| Medium      | 2 px      |
| Heavy       | 2 px      |
| Large       | 2 px      |
| Elite       | 2–3 px    |
| Mini-Boss   | 2–3 px    |
| Boss        | 3–4 px    |
| Player      | 2 px      |

Bounding box ≠ silhouette edge. Sprites should not normally touch edges.

### Enemy Visual Design
Base structure: symmetrical mechanical foundation (Steel ramp).

Infection (Magenta ramp) — **purely visual, no gameplay effect**:
- Asymmetrical organic growth that distorts silhouette, not just recolors.
- Grows stronger per biome.

Infection coverage guideline:
| Tier      | Coverage |
|-----------|----------|
| Small     | 5–10%    |
| Medium    | 10–20%   |
| Heavy     | 20–30%   |
| Large     | 30–40%   |
| Elite     | 40–60%   |
| Boss      | 60%+ integrated |

Steel must remain visible. Enemies cannot be fully magenta.

### Silhouette Discipline
- Must read at 1× zoom.
- No micro-noise detail.
- Negative space required.
- Infection must change shape (not just color).
- No internal shade equals outline value.
If silhouette fails at 1×, redesign.

### Enemy destruction visual effect
When an enemy is destroyed (HP reduced to 0 by projectile damage, not by breach), an animated explosion sprite is rendered at the enemy's center position. The explosion sprite is a short 5-frame animation (40 ms per frame) that plays once and despawns. This provides visual feedback that the enemy was successfully killed rather than despawned silently.

### Upgrade Lane Framing
Top and bottom upgrade lanes mirror each other in height and color, and both remain gameplay-active orb lanes.

### Boundary Shield Visual
The boundary shield sprite spans vertically across the enemy lane:
- Top of shield: y = ENEMY_LANE_TOP = 43
- Bottom of shield: y = ENEMY_LANE_BOTTOM = 157 (inclusive)
- Total height: 115 px

Rendered using 3-slice vertical repeat: `top` and `bot` slices drawn at natural height (15 px each); `mid` slice (12 px) tiled to fill the remainder. The current animation frame's x-offset is applied to all slice source rects. When an explosive segment is present, the shield is recolored using an HSL color-blend shader (H+S from orange tint, L from sprite texture) to preserve shading detail.

### Border Design
Top Border: 21 px. No bottom border — the bottom upgrade lane extends to the screen edge.
Must feel part of world (biome enclosure, structural framing) — not UI bars.

### Hard Constraints
DO NOT:
- Increase resolution beyond 320×180.
- Add a 4th shade to any material.
- Use near-black for interior shading.
- Exceed 72 px boss height.
- Collapse lanes visually.

Resolution and geometry are permanently locked.

### Design Philosophy
Clarity over detail.
Structure over chaos.
Silhouette over texture.
Discipline over escalation.

If something feels cramped: adjust sprite scale or spawn density. Never increase resolution.
