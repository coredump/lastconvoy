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
- Scaling: Integer only (×2, ×3, ×4, ×5, ×6). No fractional scaling.

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
- **Small:** 1 HP. On reaching the left boundary: trigger **one** damage event and despawn.
- **Medium:** multi HP. On reaching boundary: stop, occupy a boundary slot, deal repeated damage ticks until destroyed.
- **Heavy:** ~6–8 HP. On reaching boundary: stop, occupy a boundary slot, deal repeated damage ticks until destroyed. Introduced later than Medium.
- **Large:** high HP. On reaching boundary: stop, occupy a boundary slot, deal repeated damage ticks until destroyed.
- **Mini-Boss:** ~25–40 HP. Event-based only (see §13). On reaching boundary: stop, occupy a boundary slot, deal repeated damage ticks.
- **Boss:** Reserved tier. No gameplay rules defined yet.

### Shielded enemies (additive layer)
- Some enemies spawn with a shield layer (extra HP layer that must be broken first).
- No shield regeneration.
- Shielded enemies are **additive** (do not replace normal spawns).
- Frequency increases slowly over time.

## 6. Damage, shields, death
- No instant death on contact.
- Player survivability is discrete **shield segments**.

### Shields
- Shields are external armor plates embedded into the ship sprite.
- No numeric shield counter; segments are purely visual and discrete.
- Each segment = 1 shield layer.

### Damage events
On a damage tick:
- If player has ≥1 shield segment: remove exactly one segment.
- If player has 0 shields: player dies.

Damage sources:
- Small enemy boundary arrival: 1 damage event then despawn.
- Medium/large at boundary: repeated damage ticks at fixed interval until destroyed.

**Losing shields never removes drones and never reduces offense.**

## 7. Drones & shooting model
Definition:
- Any shooter that is not the player is a **drone**.

Drone types:
- **Attached drones:** persist for the run, move with the main ship.
- **Detached / cross-lane drones:** temporary; despawn after duration (TTL).

Shot-type modifiers apply to **all** shooting entities (player + drones).

Critical orb-interaction rule:
- **Drone shots do not affect upgrade orbs in any way** (no activation HP damage, no cycling).

## 8. Boundary slots & lane jam
- Left boundary has a finite number of occupancy slots.
- Medium/large (and elites) occupy a slot on arrival.
- If all slots are full:
  - The lane **jams**: additional enemies stop moving / queue.
  - Faster enemies stack behind slower/stopped ones (no overlapping). A faster enemy that catches up to a slower one matches its effective position and waits.
  - Queued enemies do not deal boundary damage until they occupy a slot.

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
Upgrades are in 3 categories:

### (A) Defense
- Adds shields (armor segments).

### (B) Drones
- Adds/improves drones:
  - Attached drone count/cap
  - Temporary detached/cross-lane drones (duration-based)

### (C) Shot types
- Modifies projectile behavior (applies to player + drones).
- Examples: piercing, spread, fire-rate changes, damage changes.
- Keep combinations readable; avoid early combinatorial explosion.

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
- Format: RON, TOML, or JSON (pick one and stay consistent).
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
