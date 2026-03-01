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
- If viewport is portrait: keep gameplay landscape, center canvas, letterbox; **do not pause** and **do not rotate gameplay**.
- Scaling: Integer only (×1, ×2, ×3, ×4, ×5, ×6). No fractional scaling.

## 3. Lanes
Two horizontal lanes (always):
- **Top lane:** Enemies
- **Bottom lane:** Upgrades (orbs)
Upgrade lane is visually smaller than enemy lane.

Lanes are **independent systems** (separate spawn timers, separate caps).

Coordinate ranges (locked):

| Section       | Rows     | Height |
|---------------|----------|--------|
| Top Border    | 0–15     | 16 px  |
| Enemy Lane    | 16–119   | 104 px |
| Divider       | 120–123  | 4 px   |
| Upgrade Lane  | 124–163  | 40 px  |
| Bottom Border | 164–179  | 16 px  |

## 4. Player
- Fixed on left side.
- Single-axis movement (vertical only) within playfield (and between lanes).
- Auto-fires forward.
- Shoots only the lane the ship is in (cross-lane influence only via **temporary detached drones**, see §7).

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

Enemy lane height 104 px; max standard enemy height 48 px; boss hard cap 72 px.

Enemy classes:
- **Small:** 1 HP. On reaching the left boundary: winds up, triggers **one** breach event, then despawns.
- **Medium:** multi HP. On reaching boundary: winds up (0.5 s), triggers one breach event, then despawns.
- **Heavy:** ~6–8 HP. On reaching boundary: winds up (1.0 s), triggers one breach event, then despawns. Introduced later than Medium.
- **Large:** high HP. On reaching boundary: winds up (1.3 s), triggers one breach event, then despawns.
- **Mini-Boss:** ~25–40 HP. Event-based only (see §13). On reaching boundary: winds up, triggers one breach event, then despawns.
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
- The Explosive Shield upgrade is a modifier, not additive: unavailable if the player has 0 segments or if an explosive segment already exists.

### Damage events (breach model)
When an enemy completes its wind-up at the boundary, a **breach event** fires:
- If player has shield: remove exactly one segment (normal segments first; explosive last).
- If player has no shield: player dies instantly.
- The breaching enemy is despawned immediately after resolution.

All enemies deal exactly **1 breach event** — there is no multi-damage or spillover.

Damage sources:
- Any enemy kind that reaches the left boundary and completes its wind-up.

**Losing shields never removes drones and never reduces offense.**

## 7. Drones & shooting model
Definition:
- Any shooter that is not the player is a **drone**.

Drone types:
- **Attached drones:** persist for the run, move with the main ship.
- **Detached / cross-lane drones:** temporary; despawn after duration (TTL).

Shot-type modifiers apply to **all** shooting entities (player + drones).

Critical orb-interaction rules:
- **Drone shots do not affect upgrade orbs in any way** (no activation HP damage, no cycling).
- Upgrade-lane drones (cross-lane) despawn immediately on any orb activation event; they do not interact with orbs.

## 8. Boundary breach & queuing
- Only one enemy may wind up at the boundary at a time (**breach lock**).
- When an enemy reaches `BOUNDARY_X` and breach is free: it enters the `Breaching` state, its movement stops, and the breach lock engages.
- If another enemy arrives within a brief simultaneous window (≤ 0.10 s) of the first, it may also join the current breach group (rare, organic simultaneous breach).
- All other enemies that reach the boundary while locked are clamped at `BOUNDARY_X + PRE_BOUNDARY_STOP_OFFSET` (24 px) and remain `Moving`. They compress naturally behind the breaching enemy, forming a visible pressure cluster.
- After all enemies in the breach group resolve, the lock is released. The frontmost compressed enemy naturally advances to `BOUNDARY_X` and starts the next breach.
- Enemies that are staggered (knocked back) while Breaching are returned to Moving and removed from the breach group, releasing the lock early.
- **Future (P2.X):** virtual slot system — 6 vertical lanes with span-based occupancy per enemy kind (Small=1, Medium/Heavy=2, Large=3) for visual variety at the boundary.

Enemy stacking behavior (unchanged):
- Faster enemies stack behind slower/stopped ones (no overlapping). A faster enemy that catches up to a slower one matches its effective position and waits.

## 9. Upgrade lane & orb interaction (two-phase)
Orbs are interactive and require deliberate time/aim.

### Orb spawning
- Orbs spawn continuously on a timer in upgrade lane.
- Spawn rate slowly increases with time (difficulty).
- Small random offset ("predictable but not to the second").
- Enforce max active orb cap; if at cap, delay spawn.
- Enemy spawns pause during elites; **orb spawns continue during elites**.

### Orb movement
- Orbs move right → left in upgrade lane.

### Two-phase interaction
Orbs spawn as generic.

**Phase 1 — Activation**
- Orb has HP.
- Player must shoot orb until HP reaches 0.
- Shots used to reduce HP **do not cycle type**.
- Drone shots do not interact with orbs.

At HP = 0, orb becomes **activated** (clear visual state).

**Phase 2 — Type cycling (only after activation)**
- Only activated orbs can be type-cycled.
- Shooting an activated orb cycles its type.
- Only shots connected to the **main ship** can cycle type.
- Each valid hit advances to next type.

**Collect**
- Player must physically collect orb to gain the selected upgrade.

## 10. Upgrade tracks (in-run)

Upgrades are collected by physically touching an activated orb. Each OrbType is a discrete track.

### (A) Shield
- **Shield (+1 Shield Segment)**: adds one shield segment per collection. Skipped from the orb pool when shields are already at cap (3 segments). Up to 3 collections.
- **Explosive Shield** *(planned, not yet implemented)*: converts one normal segment to an explosive segment. See §6 for behavior rules.

### (B) Drones
- **Drone**: adds one attached drone. Drone fires in the same lane as the player and moves with the player. *(Collection wired; drone firing not yet implemented.)*
- Upgrade-lane drones (cross-lane, temporary) are deferred to a later expansion.

### (C) Offense — Shot modifiers
All shot modifiers apply to player shots only at present; drone interaction is tracked per modifier.

- **Damage** (3 levels): increases flat damage per hit. Level 1 = 1, Level 2 = 2, Level 3 = 3. Orb removed from pool at max level.
- **FireRate** (3 levels): decreases shot interval. Level 1 = 0.18 s, Level 2 = 0.14 s, Level 3 = 0.10 s. Orb removed from pool at max level.
- **Burst / Charge Burst** (3 levels): on a separate cooldown timer, the next shot deals double damage. Cooldown interval per level: 5.0 s, 3.5 s, 2.0 s. Orb removed from pool at max level.
- **Pierce** (3 levels): projectile passes through up to N additional enemies (level = max enemies pierced beyond the first). A shot cannot hit the same enemy more than once. Orb removed from pool at max level.
- **Stagger Shot** (1 level): on hit, instantly displaces the enemy rightward by up to 12 px. Affects Small, Medium, and Heavy only — does NOT affect Large, Elite, or Mini-Boss. Applies **at most once per enemy** (subsequent hits do not knockback). Displacement is clamped by the nearest enemy to the right in the same y-band (zero movement if already touching). Orb removed from pool at max level.

**Pool rules:**
- Orb types at max level are excluded from the spawn pool automatically.
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

### Elite variants (random per event)
- **A)** Single massive elite
- **C)** Massive elite + support enemies

### Elite behavior
- Moves right → left (approach window is DPS check).
- If not killed before boundary:
  - Occupies boundary slot and deals repeated damage ticks (like large/medium).
- Elite scaling is time-only.
- Can be pushed back 24 px by an explosive shield detonation, **unless** currently occupying a boundary slot.

After elite death:
- Resume normal enemy spawning.
- Apply a small global scaling bump consistent with controlled growth.

## 13. Mini-Boss Events

Purpose: rare punctuation events distinct from Elite events.

### Timing
- Separate time-based interval from elite events with a small random offset.

### Spawn behavior
- Pause normal enemy spawning during the event.
- Spawn one Mini-Boss in the enemy lane.
- Upgrade orbs continue spawning.

### Mini-Boss behavior
- Moves right → left (approach window is a DPS check).
- If not killed before boundary: occupies a boundary slot and deals repeated
  damage ticks (like Large/Heavy).
- Mini-Boss HP scales with time (time-based only, no rubber-banding).
- Can be pushed back 24 px by an explosive shield detonation, **unless** currently occupying a boundary slot.

### After Mini-Boss death
- Resume normal enemy spawning.
- Apply a small global scaling bump consistent with controlled growth.

## 14. Inputs (gameplay requirements)
- Single-axis movement only.
- Default bindings: **W/S**, Up/Down arrows, controller stick **Y**, D-pad Up/Down.
- Provide a single setting: **Rotate Input**:
  - When enabled: A/D, Left/Right arrows, stick **X**, D-pad Left/Right.
- Opposing inputs cancel; analog deadzone applies.

## 15. Touch controls & menus
- Touch movement: vertical touch strip on left; drag up/down; release stops.
- All menus/settings must be fully operable via touch (no keyboard-only flows).

## 16. Meta progression & story (phase-gated)
- Meta progression is not part of MVP.
- When implemented (Phase 3+), meta can:
  - Increase starting stats
  - Unlock new upgrade types
  - Increase drop frequency
  - Add optional difficulty modifiers for better rewards
- Story is Phase 4+ and must not reshape core mechanics.

## 17. Tuning constants & runtime configuration
- All tuning constants and scaling curves must have **compile-time defaults** in `src/config.rs`.
- Avoid magic numbers scattered across systems.

### Runtime config file
- The game must load tuning values from an **external config file** at startup.
- The file must be human-readable and editable with a text editor — no rebuild required to change values.
- Format: TOML (human-readable, supports comments, stable parsing).
- If the file is missing or malformed, fall back to compile-time defaults silently (no crash).
- The file should cover at least: player speed, fire rate, enemy speeds/HP, spawn intervals, scaling curves, boundary slot count, orb caps/HP, elite/mini-boss intervals, shield starting count.
- Changes to the file take effect on next game launch (hot-reload is not required).

### In-game configuration screen
- The game must provide a configuration/settings screen accessible from the title screen.
- The settings screen must expose key tuning values for adjustment without editing the file.
- Any values changed in the settings screen must be written back to the config file so they persist across sessions.
- The settings screen must be fully touch-operable (consistent with §15).
- Minimal viable settings for MVP: player speed, starting shields, enemy spawn rate modifier, fire rate. Expand in later phases.

## 18. Tooling pointers (non-authoritative)
- The repo uses macroquad + Rust + Cargo.
- Native builds for development; WASM (wasm32-unknown-unknown) for browser release.
- Prefer official macroquad docs and examples:
  - https://macroquad.rs
  - https://github.com/not-fl3/macroquad

## 19. Visual Specification

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

### Divider Design
Height: 4 px. Must feel intentional — energy rail, shield barrier, or mechanical seam.
Never a flat single-color strip.

### Border Design
Top and Bottom Borders: 16 px each.
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
