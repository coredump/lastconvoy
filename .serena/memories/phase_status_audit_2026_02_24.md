# Phase Status Audit — 2026-02-24

## Summary
Codebase is at approximately P1.7 complete, with P1.8 (orbs two-phase) mostly done structurally but untested. P1.6 shields implementation is simplified (bool instead of Vec<ShieldSegment>).

## Completed Tasks
- **P1.0 Scaffolding**: All modules exist (config.rs, game.rs, input.rs, player.rs, enemy.rs, projectile.rs, orb.rs, drone.rs, shield.rs, upgrade.rs, elite.rs, boundary.rs, render.rs, sprite.rs)
- **P1.1 Rendering & scaling**: RenderTarget at 320×180, integer scaling, letterbox implemented in render.rs
- **P1.2 Input**: 1D input system (W/S/Up/Down) with gamepad + touch (draft) in input.rs
- **P1.3 Player**: Player struct with auto-fire, projectile spawning, vertical movement
- **P1.4 Lane visuals**: Background lanes, divider, borders drawn in game.rs draw_background()
- **P1.5 Enemies**: Small/Medium/Heavy/Large spawning with continuous timer, HP, speed, movement right→left, projectile collision, boundary arrival, shielded enemy layer (enemy.shield_hp)
- **P1.6 Shields & death**: Implemented as single bool (shield_active) not Vec<ShieldSegment>. On damage, removes shield or kills. Reset on key press.
- **P1.7 Boundary slots & jam**: Boundary struct with occupy_slot/release_slot, queued vs. slotted enemies, promotion logic

## Partially Implemented / In Progress
- **P1.8 Upgrade orbs (two-phase)**: Orb struct with OrbPhase::Inactive/Active, two-phase take_hit() logic (phase 1 depletes HP, phase 2 cycles type). Orb spawning, movement, projectile-orb collision, Active-only collection all implemented. **Likely structurally complete, needs testing.**
- **P1.6 Shields (variant)**: ShieldSegment struct exists (shield.rs) but not used by GameState. GameState uses simple bool. Mismatch.

## Not Yet Implemented
- **P1.9 Upgrade tracks**: Orb cycling works structurally but no actual upgrade application (no Defense, Drone, ShotType effect code)
- **P1.10 Drone system**: Drone struct exists; not actively fired or positioned in game loop
- **P1.11 Time-based scaling**: Enemy spawn scaling exists; orb scaling, orb intro time, elite intervals need verification
- **P1.12 Elite events**: EliteEvent struct created but not wired into enemy spawning pause/resume
- **P1.13 Mini-Boss events**: Stub (miniboss_timer exists, no spawn logic)
- **P1.14 MVP polish**: Some particle/flash feedback may exist in drawing; check sprite animation

## Discrepancies
1. **Shield model mismatch**: SPEC §6 says "discrete shield segments" visually. P1.6 should use Vec<ShieldSegment> tracking active count. Currently only bool.
2. **Drone firing not wired**: Drones are cleared on reset but never updated, fired, or positioned relative to player.
3. **Orbs may not be tested**: Two-phase logic looks correct but no gameplay integration verified (actual run).

## Action Items for Next Session
1. Verify P1.8 orbs work in practice (run game, check two-phase behavior)
2. Fix P1.6 shields: replace bool with Vec<ShieldSegment>, update take_player_damage to remove one segment
3. Implement P1.9 upgrade application (Defense: add segment; Drone: add drone; ShotType: apply piercing modifier)
4. Implement P1.10 drone firing and positioning
5. Wire up P1.12 elite pausing and resumption
6. Implement P1.13 mini-boss spawning

## Git Status (as of sync start)
- Latest commit: 797114e "feat: up to 1.5 finished"
- Main branch, no uncommitted changes beyond modified doc files
