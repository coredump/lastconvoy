# Config Format Switch — 2026-02-26

## Summary
Code has switched from RON to TOML for runtime config file format. All documentation has been updated to reflect this.

## Changes Made
1. **CLAUDE.md**:
   - Line 13: Removed RON from Context7 plugin description, replaced with TOML
   - Lines 58-59: Updated config file references from `config.ron` (serde + RON) to `config.toml` (serde + TOML)

2. **SPEC.md**:
   - Line 94: Updated config file reference from `config.ron` to `config.toml`
   - Line 292: Changed format guidance from "RON, TOML, or JSON (pick one)" to "TOML (human-readable, supports comments, stable parsing)"

3. **TASKS.md**:
   - Lines 39-41: Updated P1.0 config loading task to specify TOML format and `config.toml` filename

## Code Implementation Status
- ✓ Cargo.toml includes `toml = "0.8"` dependency
- ✓ src/config.rs has `load_runtime_config()` using `toml::from_str()` to parse `config.toml`
- ✓ config.toml file exists at project root
- ✓ Silent fallback to defaults if config file missing/malformed

## Side Note: Orb Y Centering
Also verified this session:
- Orb spawn Y is properly centered in upgrade lane: `lane_mid = (UPGRADE_LANE_TOP + UPGRADE_LANE_BOTTOM + 1) / 2.0`
- Formula subtracts ORB_H/2 to center the sprite: `y = lane_mid - ORB_H / 2.0`
- This is correct for 20-pixel orb in 40-pixel upgrade lane (rows 124–163)
