# lastconvoy — Suggested Commands

## Development
```bash
cargo run                    # run native dev build
cargo run --release          # run native release build
```

## Before every commit
```bash
cargo fmt                              # format (required)
cargo clippy -- -W clippy::all         # lint (must pass clean)
```

## Tests (Phase 2+ only)
```bash
cargo test
```

## WASM release build
```bash
cargo build --target wasm32-unknown-unknown --release
```

## Utilities
```bash
git status
git diff
git log --oneline -10
ls src/
```
