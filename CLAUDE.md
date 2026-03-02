# CLAUDE.md

## Doc hierarchy
- **Gameplay truth:** `SPEC.md` — read before any gameplay question.
- **Task order:** `TASKS.md` — read before starting or resuming work.
- **This file:** tooling rules, repo conventions, constraints.
- Conflicts: SPEC wins gameplay, this file wins process, TASKS wins order.

## Plugins & MCPs

### Plugins (auto-loaded via claude-plugins-official)
- **Serena** — code navigation via sub-agent. Use for: finding definitions, references, file structure, symbol lookups. Do NOT memorize project structure; query Serena instead.
- **Context7** — library docs via sub-agent. Use for: macroquad API, serde, TOML, any crate docs. Do NOT guess at APIs; query Context7 instead.
- **feature-dev** — structured 7-phase dev workflow. Use `/feature-dev` when starting a new task from TASKS.md. Spawns sub-agents for exploration, design, and review — keeps main context lean.
- **security-guidance** — passive hook on PreToolUse. Blocks dangerous commands automatically. No action needed.

### MCPs
- **memory** — persistent knowledge graph (`.claude/memory.json`). Store architecture decisions, resolved ambiguities, and session learnings here. Check memory at session start before re-reading files.

### Rules
- Prefer plugin/MCP lookups over reading files or guessing. This saves context.
- **contextplus when better** — prefer contextplus tools over alternatives when they provide a clearer or more direct answer: use `get_blast_radius` instead of grep for cross-file usages; use `semantic_code_search` / `semantic_identifier_search` for intent-based searches; use `run_static_analysis` for inline cargo checks; use `get_context_tree` / `get_file_skeleton` for structural overviews. Fall back to Serena + ripgrep for precise symbol edits and exact-match searches.
- **Serena & ripgrep MCPs first** — always use these for code navigation, file search, and content search before Bash/shell tools.
- **Context7 first** — always query Context7 for library docs before relying on training knowledge.
- **commit-commands skills first** — use `/commit-commands:commit` for git commits. Commit directly to main (no PRs/branches for now).
- Store important decisions and resolved questions in **memory** so future sessions don't re-derive them.
- Use `/feature-dev` for any task that touches multiple files or introduces a new system.
- Use `/compact` between tasks to shed old context.
- **Always update CLAUDE.md** when user states a new preference, rule, or workflow for tool usage — so it persists across sessions.
- **Always sync after any change** — after ANY code or design change, immediately update: (1) memory MCP with decisions/findings, (2) SPEC.md if gameplay rules changed, (3) TASKS.md if task status changed, (4) CLAUDE.md if architecture/status/conventions changed. Do not defer. Keep sync state always current.

### MCP usage — detailed rules
- **Priority order:** (1) contextplus when it's the best tool (blast radius, semantic search, static analysis, structural overview), (2) Serena + ripgrep for precise symbol lookup/editing and exact-match search, (3) Context7 for API docs, (4) memory MCP for decisions/state, (5) filesystem MCP for file reads/writes/directory ops, (6) Bash only as last resort.
- **Serena** — code navigation only. Use `find_symbol`, `find_referencing_symbols`, `get_symbols_overview`, `search_for_pattern`. Do NOT use Serena `write_memory`/`read_memory` — use memory MCP instead.
- **memory MCP** — primary persistent store. Call `read_graph` at session start. MEMORY.md is a concise index only; details live in the graph. Always store architecture decisions, resolved ambiguities, and key findings here.
- **Context7** — always call `resolve-library-id` first, then `query-docs`. Never skip for macroquad/crate API questions.
- **ripgrep MCP** — prefer `mcp__ripgrep__search` / `advanced-search` over `grep`/`rg` in Bash.
- Do NOT memorize project structure across sessions — query Serena fresh each time.
- Do NOT read files speculatively — use symbolic tools to retrieve only what's needed.
- Do NOT re-derive decisions already in the memory graph — check it first.

### contextplus — strict workflow rules
- **`get_context_tree` at every task start — mandatory, no exceptions.** Run before any other exploration.
- **`get_file_skeleton` before reading any unfamiliar file.** Skip only when the exact symbol/line is already known.
- **`get_blast_radius` before deleting or modifying any symbol.** Never remove code without checking impact first.
- **`run_static_analysis` after writing any code.** Catch unused imports, dead code, type errors before moving on.
- **Batch independent tool calls in parallel.** Never make sequential calls that could run simultaneously.
- **`propose_commit` is SKIPPED as a file-writing tool** — use filesystem MCP / Edit / Write instead. Its formatting rules apply to NEW files only (do not retroactively reformat existing Rust files).

### contextplus — new file formatting rules (apply to NEW files only)
- Every new file starts with a 2-line header comment: line 1 = file purpose, line 2 = key dependencies or blank.
- No inline comments. Logic must be self-evident from naming; add a header comment if explanation is needed.
- Code ordering within a file: imports → enums → type aliases → constants → functions.
- Abstraction thresholds: inline code used once and <20 lines; extract to function if >30 lines or used 2+ times.

### contextplus — anti-patterns (STRICT — never do these)
- Never read a full file without calling `get_file_skeleton` first.
- Never delete or rename a symbol without calling `get_blast_radius` first.
- Never leave unused imports or variables after a refactor.
- Never retry a failing operation in a loop — diagnose root cause or ask the user.

## Toolchain
- Rust (stable), Cargo, macroquad, serde + toml, rustfmt, clippy.
- Native builds for dev (`cargo run`), WASM for release (`cargo build --target wasm32-unknown-unknown --release`).

## Commands
```bash
cargo run                          # dev
cargo run --release                # release native
cargo fmt                          # format (run before commit)
cargo clippy -- -W clippy::all     # lint (must pass before commit)
cargo test                         # tests (Phase 2+ only)
cargo build --target wasm32-unknown-unknown --release  # WASM
```

## Non-negotiables (do not look these up — memorize)
- Internal resolution: 320×180, landscape always, integer scaling only.
- Single-axis player movement (vertical).
- Four fixed vertical bands: top border (0–20), top upgrade lane (21–42), enemy lane (43–157), bottom upgrade lane (158–179). No bottom border.
- Time-based scaling only. No rubber-banding.
- Orbs: two-phase (activate THEN collect). Type is fixed at spawn; no cycling mechanic.
- Drone shots interact with orbs equally to player shots (activation only; no cycling mechanic).
- Boundary uses breach-lock + wind-up + re-breach cooldown (no slot occupancy model). Enemies stack behind slower/stopped ones (no overlapping). `BOUNDARY_X` = 36.0 (in front of player), `PLAYER_X` = 8.0.
- EnemyElite1 and MiniBoss: event-only, never in regular spawn pool.

## Architecture
- No ECS. Plain structs + `Vec<T>` + `GameState`.
- Frame-rate independent: all movement/timers use `get_frame_time()`.
- Render pipeline: draw to 320×180 `RenderTarget`, blit to screen with integer scale + letterbox.
- Entity pools: `Vec<T>` with `retain()` or swap-remove. Pre-allocate hot paths.

## Config (two layers)
- Compile-time defaults: `src/config.rs`.
- Runtime overrides: `config.toml` at project root (serde + TOML). Silent fallback if missing/malformed.
- Settings screen (Phase 2) writes back to `config.toml`.

## Conventions
- One concern per module. Keep `main.rs` thin.
- Naming: `Player`, `Enemy`, `EnemyKind`, `Orb`, `Drone`, `ShieldSegment`, `EliteEvent`.
- Commit messages: `P1.X: brief description`.
- Text font policy: `ui_font` uses **AtariGames** (`assets/fonts/atarigames/atarigames-bitmap.{png,json}`); Monogram assets remain at `assets/fonts/monogram/`; `logo_font` uses **Edunline** (`assets/fonts/edunline/edunline-bitmap.{png,json}`). AB-test alternates (GravityBold8, LowIndustrial) remain at `assets/fonts/gravity/` and `assets/fonts/lowindustrial/`. GravityRegular5 bitmap also at `assets/fonts/gravity/gravityregular5-bitmap.{png,json}`.
- No `unsafe` unless unavoidable.
- No tests in Phase 1. Tests allowed Phase 2+.

## Phase 1 status
P1.0–P1.7 COMPLETE. P1.8 (orbs two-phase) STRUCTURALLY COMPLETE, pending gameplay verification. P1.9 UPDATED (offense tracks converted from permanent levels to temporary refreshable buffs; Explosive core logic implemented with polish/verification pending). P1.10 DONE (drone system fully implemented; Drone orb in normal pool). P1.14 COMPLETE (explosion effect on enemy death, title/pause screens, upgrade HUD redesign with drone placeholder + vertical timer bars, touch controls marked broken for P2.0 redesign). P1.15 UI POLISH (2026-03-02): per-frame animation durations, monogram_font + logo_sprite added, title screen redesign with logo sprite, floating text font switched to monogram, run timer centered in top bar.

Recent additions (balance + UI + state management):
- **Offense buff model**: Damage/FireRate/Burst/Pierce/Stagger are now temporary buffs with per-type durations and fixed magnitudes; collecting an active buff refreshes timer (no tier stacking).
- **HP scaling**: tripled (0.001 → 0.003 per second); heavy/large multipliers increased.
- **Orb pool gating**: active offense buff types are excluded from spawn pool until expiry (Shield/Explosive/Drone rules unchanged).
- **Spawn pressure**: big-enemy inject coverage slack +0.10 over target; cadence ramp accelerates up to 25% faster by 10 min.
- **UI/HUD systems** (new `src/text.rs` module): BitmapFont bitmap text renderer (Monogram assets), floating upgrade-name text, upgrade icon HUD, run timer HUD, game over screen with time/kills/breaches stats.
- **Balance telemetry**: `dps_estimate()` + `large_ttk()` logged every 30 s; kills/breaches counters tracked in `GameState`.
- **Log analysis**: debug log now appends to file and emits `RUN_START`/`RUN_END` markers; `scripts/analyze_balance_log.sh` parses multiple runs in one log and outputs per-run + aggregate reports (`--last`, `--run N`, `--no-aggregate`).
- **Lane crossing gate**: shots are blocked by invisible 1px barriers at enemy/upgrade boundaries except through a left-side gate corridor; DroneRemote pickup now spawns one remote drone per upgrade lane (top sprite mirrored).
- **Explosive shield HSL tint**: `color_blend_material` (GLSL ES HSL shader) in `GameState` recolors shield by taking H+S from orange tint and L from sprite texture, preserving shading (Aseprite "Color" blend mode). `Sprite::draw_3slice_vertical_hsl()` wraps the draw with material bind/unbind.
- **config.toml baked at compile time**: `BAKED_CONFIG = include_str!("../config.toml")` in `config.rs`; release/WASM builds use these as defaults. Native dev still layers runtime overrides on top.
- **WASM compat**: `debug_log.rs` fs ops are cfg-gated on `not(target_arch = "wasm32")`; `window_conf()` sets `WebGLVersion::WebGL2`; vite COOP/COEP headers commented out.
- **upgrade_track sprite**: asset added (`assets/sprites/objects/upgrade_track.json/png`), loaded in `main.rs`, passed to `GameState`. Slices: `front` (40×21) and `rail` (10×21).
- **is_burst removed**: `Projectile.is_burst` field and burst-specific draw path removed; burst shots use the same sprite/color as normal shots.
- **Orb spawn sync**: each orb spawn tick now attempts spawns on both upgrade lanes simultaneously, with independent per-lane type rolls and a global orb cap fallback when only one slot remains.
- **Boundary shield 3-slice rendering**: `sprite.rs` now parses Aseprite JSON `slices` into `Sprite.slices: HashMap<String, Rect>` and exposes `draw_3slice_vertical()`. Shield drawn at `BOUNDARY_X - 3.0`, spans y=43–157 (115 px, enemy lane only). Slice data: top h=15, mid h=12, bot h=15. Also exposes `draw_clipped_h()` for partial-height tile drawing.
- **Rail wall background**: `rail_wall` animated sprite (36×36, single frame) tiled vertically in the player column (x=0, y=21–158), drawn first in `draw_background()` so it sits behind all entities. Boundary marker line removed.
- **Orb despawn fix**: orbs now only despawn when fully off the left screen edge (`o.x + o.width <= 0`), not when passing the player column.
- **Explosion effect** (P1.14, 2026-03-01): `Explosion` struct in `GameState` (x, y, timer); spawned at enemy center on death; drawn as `explosion_2` sprite (5 frames, 40ms per frame).
- **Title & pause screens** (P1.14, 2026-03-01): `at_title` and `paused` flags in `GameState`; any key starts game from title; P+ESC toggles pause overlay with controls list and game name; pause state gates gameplay updates.
- **Upgrade HUD redesign** (P1.14, 2026-03-01): Shield/Explosive slots removed from HUD; drone slot shown as permanent placeholder (Upgrade Teal Very Dark box when inactive); expiring buff icons (Damage, FireRate, Burst, Pierce, Stagger) only shown when active; vertical timer bars (2px wide, Upgrade Teal Light, shrinks downward); new `top_bar` sprite asset for background.
- **Sprite::tile_w/tile_h fields** (2026-03-01): added to `Sprite` struct for frame-rect computation support for animated explosions.
- **Touch controls flagged** (2026-03-01): current touch input marked broken and not ready for use; SPEC §15 updated; P2.0 task created for touch redesign.
- **Release workflow rule** (v0.1.0): always bump `Cargo.toml` version first, commit, then tag+push+GitHub release with changelog from previous tag.
- **Per-frame animation durations** (2026-03-02): `sprite.rs` now stores `frame_durations: Vec<f32>` (per-frame) + `tag_frame_offset: Vec<u32>` instead of single `tag_frame_dur`. Frames sorted alphabetically in from_json(). Animation loop reads durations from per-frame index.
- **Monogram font and logo sprite** (2026-03-02): New GameState fields `monogram_font: BitmapFont` (from `assets/fonts/monogram/monogram-bitmap.{png,json}`) and `logo_sprite: Sprite` (from `assets/sprites/ui/logo.json`, 191×21 per frame, 7-frame looping animation). Loaded in main.rs, passed to GameState::new(). Animation enabled with per-frame durations (frame 0=1000ms, others 16-2000ms).
- **Title screen redesign** (2026-03-02): Replaced text title with logo_sprite drawn at centered x, y=30. Subtitle 'LAST CONVOY DEFENSE' positioned dynamically at y=logo_y+sprite_height+5px (currently y=56). Controls list moved to y=78.
- **Floating text font** (2026-03-02): Upgrade pickup text now uses monogram_font (lighter appearance) instead of ui_font.
- **Run timer centering** (2026-03-02): Timer now vertically centered in top bar: y = ((TOP_BORDER_BOTTOM - TOP_BORDER_TOP + 1) as f32 - size.y) * 0.5 + 2.0.
- **Logo animation re-enabled** (2026-03-02): Logo sprite animation now active with per-frame durations. Asset updates applied to fix color persistence artifact in frame transitions. logo_sprite.update(dt) called in at_title state.
- **Logo dimensions dynamic** (2026-03-02): Sprite tile_w/tile_h (191×21) read at runtime; subtitle y-position auto-adjusts based on sprite.tile_h. No hardcoded dimensions in code.

Source files now include: `main.rs`, `config.rs`, `game/` (mod.rs, game_buff.rs, game_combat.rs, game_draw.rs, game_orb.rs, game_spawn.rs), `player.rs`, `enemy.rs`, `projectile.rs`, `orb.rs`, `drone.rs`, `shield.rs`, `upgrade.rs`, `elite.rs`, `boundary.rs`, `input.rs`, `render.rs`, `debug_log.rs`, `text.rs`, `sprite.rs`.

Next priorities: (1) Verify P1.8 and explosive shield gameplay feel in practice; (2) Verify P1.10 drone behavior in-game; (3) Continue Phase 1 scaling/event verification in `TASKS.md`.

## Before writing code
1. Check **memory** for prior decisions and context on this area.
2. Read relevant section of `SPEC.md` for gameplay rules.
3. Check `TASKS.md` for current task and dependencies.
4. Use **Context7** for any macroquad/crate API questions.
5. Use **Serena** to find existing code before creating new files.
6. For non-trivial tasks: use `/feature-dev` to get the structured workflow.
7. After completing a task: store key decisions in **memory**, then `/compact`.
