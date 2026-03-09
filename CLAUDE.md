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
- **claude-mem** (plugin) — persistent cross-session memory and smart code exploration. Use `mcp__plugin_claude-mem_mcp-search__smart_search` to recall prior decisions. Skills: `/claude-mem:mem-search`, `/claude-mem:smart-explore`, `/claude-mem:make-plan`, `/claude-mem:do`.
- **ripgrep** — content search. Prefer `mcp__ripgrep__search` / `advanced-search` over Bash grep/rg.
- **filesystem** — file reads/writes/directory ops. Use before falling back to Bash.

### Skills (invoked via `/skill-name`)
- **feature-dev** — structured 7-phase dev workflow. Use `/feature-dev` when starting a new task from TASKS.md.
- **commit-commands** — `/commit`, `/commit-push-pr`, `/clean_gone`. Use for all git operations.
- **claude-code-setup** — automation recommendations.
- **claude-mem** — `/claude-mem:mem-search` (search persistent memory), `/claude-mem:smart-explore` (AST-based code exploration), `/claude-mem:make-plan` (plan multi-step tasks), `/claude-mem:do` (execute plans via subagents).

### Hooks
- **security-guidance** — passive PreToolUse blocker. No action needed.

### Rules
- Prefer MCP/tool lookups over reading files or guessing. This saves context.
- **Serena & ripgrep MCPs first** — always use these for code navigation, file search, and content search before Bash/shell tools.
- **Context7 first** — always query Context7 for library docs before relying on training knowledge.
- **commit-commands skills first** — use `/commit-commands:commit` for git commits. Commit directly to main (no PRs/branches for now).
- Store important decisions and resolved questions in **claude-mem** so future sessions don't re-derive them. Use `/claude-mem:mem-search` to check before re-deriving.
- Use `/feature-dev` for any task that touches multiple files or introduces a new system.
- Use `/compact` between tasks to shed old context.
- **Always update CLAUDE.md** when user states a new preference, rule, or workflow for tool usage — so it persists across sessions.
- **Always sync after any change** — after ANY code or design change, immediately update: (1) claude-mem with decisions/findings, (2) SPEC.md if gameplay rules changed, (3) TASKS.md if task status changed, (4) CLAUDE.md if architecture/status/conventions changed. Do not defer. Keep sync state always current.

### MCP priority order
(1) Serena + ripgrep for symbol lookup/editing and content search, (2) claude-mem for persistent decisions and code exploration, (3) Context7 for API docs (`resolve-library-id` first, then `query-docs`), (4) filesystem MCP for file reads/writes, (5) Bash as last resort.

### Workflow rules
- Use `get_symbols_overview` or `smart_outline` before reading files. Use `find_referencing_symbols` before deleting/modifying symbols.
- Run `cargo clippy` after writing code. Batch independent tool calls in parallel.
- Don't memorize project structure — query Serena fresh. Don't re-derive decisions — check claude-mem first.
- New files: 2-line header comment, no inline comments, ordering (imports→enums→types→constants→functions).
- Keep changes small. Cap retries to 1-2. Diagnose failures, don't loop.

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
- `EnemyKind::XL` (renamed from Elite): regular enemy, DeepSpace biome only. Boss placeholder fires at end of every biome. No mini-boss or elite event timers.

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
- Naming: `Player`, `Enemy`, `EnemyKind`, `Orb`, `Drone`, `ShieldSegment`.
- Commit messages: `P1.X: brief description`.
- Text font policy: `ui_font` uses **AtariGames** (`assets/fonts/atarigames/atarigames-bitmap.{png,json}`); Monogram assets at `assets/fonts/monogram/`; `logo_font` uses **Edunline** (`assets/fonts/edunline/edunline-bitmap.{png,json}`). AB-test alternates (GravityBold8, LowIndustrial) at `assets/fonts/gravity/` and `assets/fonts/lowindustrial/`.
- No `unsafe` unless unavoidable.
- No tests in Phase 1. Tests allowed Phase 2+.

## Status
- **Phase 1**: P1.0–P1.20 DONE. Remaining: gameplay verification (P1.8 orbs, P1.9b explosive, P1.11 scaling).
- **Phase 3**: P3.0–P3.3 DONE (persistence, upgrade shop, game-over screen overhaul, meta points economy). Next: P3.4 save slots.
- See `TASKS.md` for full task details and future phases.

## Story & Meta (Phase 4+)
- **STORY.md** (2026-03-04): Full lore, premise, characters (Custodian, Rael, Voss, The Entity), four biomes (Infected Atmosphere, Low Orbit, Outer System, Deep Space Corridor), tone, story beat structure. Story panel art started at `art/story/base.aseprite`.

## Before writing code
1. Check **claude-mem** (`/claude-mem:mem-search` or `mcp__plugin_claude-mem_mcp-search__smart_search`) for prior decisions and context on this area.
2. Read relevant section of `SPEC.md` for gameplay rules.
3. Check `TASKS.md` for current task and dependencies.
4. Use **Context7** for any macroquad/crate API questions.
5. Use **Serena** to find existing code before creating new files.
6. For non-trivial tasks: use `/feature-dev` to get the structured workflow.
7. After completing a task: store key decisions in **claude-mem**, then `/compact`.
