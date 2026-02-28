# lastconvoy — Task Completion Checklist

After completing any task:

1. `cargo fmt` — format all source files
2. `cargo clippy -- -W clippy::all` — must pass with zero warnings/errors
3. `cargo run` — verify the game runs without panics or regressions
4. Store key decisions/resolved questions in the Claude memory MCP (`mcp__memory__create_entities`)
5. Update `TASKS.md` if task status changed (or note it in memory)
6. Commit with message format: `P1.X: brief description`
7. Use `/compact` to shed context before starting the next task
