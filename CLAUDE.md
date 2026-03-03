# CLAUDE.md

## Doc hierarchy
- **Gameplay truth:** `SPEC.md` — read before any gameplay question.
- **Task order:** `TASKS.md` — read before starting or resuming work.
- **This file:** tooling rules, repo conventions, constraints.
- Conflicts: SPEC wins gameplay, this file wins process, TASKS wins order.

## Tools

### MCP Servers (provide tools)
- **Serena** (plugin) — code navigation. Use `find_symbol`, `find_referencing_symbols`, `get_symbols_overview`, `search_for_pattern`. Do NOT memorize project structure; query Serena instead.
- **Context7** (plugin) — library docs. Use for: macroquad API, serde, TOML, any crate docs. Always call `resolve-library-id` first, then `query-docs`. Do NOT guess at APIs.
- **contextplus** (project `.mcp.json`) — semantic search, blast radius, static analysis, structural overview. Prefer over alternatives when it's the clearest tool.
- **memory** — persistent knowledge graph (`.claude/memory.json`). Store architecture decisions, resolved ambiguities, and session learnings here. Call `read_graph` at session start.
- **ripgrep** — content search. Prefer `mcp__ripgrep__search` / `advanced-search` over Bash grep/rg.
- **filesystem** — file reads/writes/directory ops. Use before falling back to Bash.

### Skills (invoked via `/skill-name`)
- **feature-dev** — structured 7-phase dev workflow. Use `/feature-dev` when starting a new task from TASKS.md.
- **commit-commands** — `/commit`, `/commit-push-pr`, `/clean_gone`. Use for all git operations.
- **claude-code-setup** — automation recommendations.

### Hooks
- **security-guidance** — passive PreToolUse blocker. No action needed.

### Rules
- Prefer MCP/tool lookups over reading files or guessing. This saves context.
- **contextplus when better** — prefer contextplus tools when they provide a clearer answer: use `get_blast_radius` instead of grep for cross-file usages; use `semantic_code_search` / `semantic_identifier_search` for intent-based searches; use `run_static_analysis` for inline cargo checks; use `get_context_tree` / `get_file_skeleton` for structural overviews. Fall back to Serena + ripgrep for precise symbol edits and exact-match searches.
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

### Execution rules
1. Think less, execute sooner: make the smallest safe change that can be validated quickly.
2. Do not serialize 10 independent commands; batch parallelizable reads/searches.
3. If a command fails, avoid blind retry loops. Diagnose once, pivot strategy, continue.
4. Cap retry attempts for the same failing operation to 1-2 unless new evidence appears.
5. Keep outputs concise: short status updates, no verbose reasoning dumps.

### Token-efficiency rules
1. Treat 100 effective tokens as better than 1000 vague tokens.
2. Use high-signal tool calls first (`get_file_skeleton`, `get_context_tree`, `get_blast_radius`).
3. Read full file bodies only when signatures/structure are insufficient.
4. Avoid repeated scans of unchanged areas.
5. Prefer direct edits + deterministic validation over extended speculative analysis.

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
cargo deny check                   # license + advisory check
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
- Compile-time defaults: `src/config.rs`. `BAKED_CONFIG = include_str!("../config.toml")` for release/WASM.
- Runtime overrides: `config.toml` at project root (serde + TOML). Silent fallback if missing/malformed.
- Settings screen (Phase 2) writes back to `config.toml`.

## Conventions
- One concern per module. Keep `main.rs` thin.
- Naming: `Player`, `Enemy`, `EnemyKind`, `Orb`, `Drone`, `ShieldSegment`, `EliteEvent`.
- Commit messages: `P1.X: brief description`.
- Text font policy: `ui_font` uses **AtariGames** (`assets/fonts/atarigames/atarigames-bitmap.{png,json}`); Monogram assets at `assets/fonts/monogram/`; `logo_font` uses **Edunline** (`assets/fonts/edunline/edunline-bitmap.{png,json}`). AB-test alternates (GravityBold8, LowIndustrial) at `assets/fonts/gravity/` and `assets/fonts/lowindustrial/`.
- No `unsafe` unless unavoidable.
- No tests in Phase 1. Tests allowed Phase 2+.

## Phase 1 status
- **P1.0–P1.7**: COMPLETE.
- **P1.8** (orbs two-phase): STRUCTURALLY COMPLETE, pending gameplay verification.
- **P1.9** (offense buffs): UPDATED — temporary refreshable buffs with per-type durations; Explosive core implemented, polish/verification pending.
- **P1.10** (drone system): DONE — fully implemented; Drone orb in normal pool.
- **P1.14** (explosion FX, title/pause screens, HUD redesign, touch flagged): COMPLETE.
- **P1.15** (UI polish 2026-03-02): DONE — per-frame animation durations, monogram_font + logo_sprite, title screen logo, floating text font, run timer centering.
- **P1.16** (screen flash on shield loss 2026-03-03): DONE — brief red full-screen overlay on any shield hit via `FlashEffect`; `screen_flash` field on `GameState`.

Source files: `main.rs`, `config.rs`, `game/` (mod.rs, game_buff.rs, game_combat.rs, game_draw.rs, game_orb.rs, game_spawn.rs), `player.rs`, `enemy.rs`, `projectile.rs`, `orb.rs`, `drone.rs`, `shield.rs`, `upgrade.rs`, `elite.rs`, `boundary.rs`, `input.rs`, `render.rs`, `debug_log.rs`, `text.rs`, `sprite.rs`.

Next priorities: (1) Verify P1.8 and explosive shield gameplay feel in practice; (2) Verify P1.10 drone behavior in-game; (3) Continue Phase 1 scaling/event verification in `TASKS.md`.

## Before writing code
1. Check **memory** for prior decisions and context on this area.
2. Read relevant section of `SPEC.md` for gameplay rules.
3. Check `TASKS.md` for current task and dependencies.
4. Use **Context7** for any macroquad/crate API questions.
5. Use **Serena** to find existing code before creating new files.
6. For non-trivial tasks: use `/feature-dev` to get the structured workflow.
7. After completing a task: store key decisions in **memory**, then `/compact`.
