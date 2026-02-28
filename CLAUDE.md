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
- Store important decisions and resolved questions in **memory** so future sessions don't re-derive them.
- Use `/feature-dev` for any task that touches multiple files or introduces a new system.
- Use `/compact` between tasks to shed old context.

## Toolchain
- Rust (stable), Cargo, macroquad, serde + ron, rustfmt, clippy.
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
- Two lanes: enemies top (rows 16–119), upgrades bottom (rows 124–163).
- Time-based scaling only. No rubber-banding.
- Orbs: two-phase (activate THEN cycle). Activation hits do NOT cycle.
- Drone shots NEVER interact with orbs.
- Boundary has finite slots; when full, lane jams. Enemies stack behind slower/stopped ones (no overlapping). `BOUNDARY_X` = 36.0 (in front of player), `PLAYER_X` = 8.0.
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
- No `unsafe` unless unavoidable.
- No tests in Phase 1. Tests allowed Phase 2+.

## Phase 1 status
P1.0–P1.7 COMPLETE. P1.6 shields fully implemented (Vec<ShieldSegment> done). P1.8 (orbs two-phase) STRUCTURALLY COMPLETE, needs gameplay verification. P1.9 PARTIAL (Defense orb grants shields on collection; other tracks not yet applied). HP scaling bug fixed (`.ceil()` → `.round().max(1.0)`). Next priorities: (1) Verify P1.8 orbs work in practice; (2) Implement remaining P1.9 upgrade effects (Speed, Offense, Drone); (3) Implement P1.10 drone firing. See `TASKS.md` for details.

## Before writing code
1. Check **memory** for prior decisions and context on this area.
2. Read relevant section of `SPEC.md` for gameplay rules.
3. Check `TASKS.md` for current task and dependencies.
4. Use **Context7** for any macroquad/crate API questions.
5. Use **Serena** to find existing code before creating new files.
6. For non-trivial tasks: use `/feature-dev` to get the structured workflow.
7. After completing a task: store key decisions in **memory**, then `/compact`.
