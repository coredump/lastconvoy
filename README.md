# Last Convoy

Minimalist systemic roguelite shooter. Endless run. You hold the line or you die.

**[Play in browser](https://coredump.github.io/lastconvoy/)** — no install required.

---

## About

Last Convoy is a lane-pressure shooter, not a bullet-hell. You move on a single vertical axis while enemies march left across the screen. Shoot them down before they breach your boundary. Activate and collect upgrade orbs from the lanes above and below to boost your weapons, add drones, or reinforce your shields.

The run never ends — it only gets harder.

- Endless scaling difficulty
- Temporary offense buffs (damage, fire rate, burst, pierce, stagger)
- Drone companions
- Boundary breach system with shield segments
- 320×180 pixel art, integer scaling

> **Note:** The roguelite meta-progression layer (between-run upgrades, unlocks, saves) is not yet implemented. What's here is the core gameplay loop.

---

## Download

Native builds for Linux, macOS (ARM), and Windows are attached to each [release](https://github.com/coredump/lastconvoy/releases).

A `dev` pre-release is updated automatically on every push to `main` if you want the latest build.

**Running the native build:**
1. Download and extract the archive for your platform
2. Run the `lastconvoy` binary from inside the extracted folder (the `assets/` directory must be next to it)
3. Edit `config.toml` to tune gameplay values — the file is self-documenting

---

## Build from source

**Prerequisites:** [Rust stable](https://rustup.rs/)

```bash
# Clone
git clone https://github.com/coredump/lastconvoy.git
cd lastconvoy

# Run (native)
cargo run

# Release build (native)
cargo run --release

# WASM build
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

**Linux system dependencies** (required for native builds):
```bash
sudo apt-get install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

After a WASM build, copy `assets/web/index.html`, `assets/web/mq_js_bundle.js`, the compiled `.wasm`, and the `assets/fonts` and `assets/sprites` directories to a directory and serve it locally.

---

## Art & tools

All original sprites were made by hand in [Aseprite](https://github.com/aseprite/aseprite). No AI image generation was used for the original art.

The parallax background and explosion sprites are from the [Ultimate Warped Collection](https://ansimuz.itch.io/ultimate-warped-collection) by [Ansimuz](https://ansimuz.itch.io/), used under its asset license.

Original sprites are licensed under [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/).

### Contributing

Art contributions are very welcome — the developer is a programmer, not an artist. If you'd like to contribute sprites, animations, or visual polish, open an issue or a PR.

---

## Tech

- [Rust](https://www.rust-lang.org/) + [macroquad](https://macroquad.rs/)
- [serde](https://serde.rs/) + TOML for runtime config
- GitHub Actions for CI, WASM deploy (GitHub Pages), and native releases

Dependency licenses are checked with [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny).

Most of the code was written with assistance from [Claude](https://claude.ai/) (Anthropic).

---

## Roadmap

Roughly in order:

- [ ] Elite and Mini-Boss event systems (spawn pause, DPS checks)
- [ ] Start screen
- [ ] Game over summary (time survived, kills, breaches)
- [ ] Settings screen (in-game config editing)
- [ ] Touch input polish for mobile browsers
- [ ] Visual polish — shield loss feedback, enemy destruction particles, elite arrival cues
- [ ] Roguelike meta-progression — saves, meta points, permanent upgrades, unlocks
- [ ] Story layer — pixel/comic panels, character unlocks, story runs

---

## Licenses

| Asset | License |
|---|---|
| Source code | [MIT](LICENSE) |
| Original sprites | [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/) |
| Monogram font (datagoblin) | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) — public domain |
| AtariGames font (Kieran) | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) — public domain, from [Nb Pixel Font Bundle](https://nimblebeastscollective.itch.io/nb-pixel-font-bundle) by nimblebeastscollective |
| Gravity Pixel Font — Bold8 + Regular5 (John Watson / jotson) | Free for personal and commercial use — [itch.io](https://jotson.itch.io/gravity-pixel-font) |
| Edunline font (Brian Kent) | Freeware — personal and commercial use permitted; no resale or modification without permission |
| Ansimuz parallax background + explosions | Personal and commercial use permitted; redistribution/resale as standalone assets not permitted |

See [`assets/LICENSE`](assets/LICENSE) for full third-party asset terms.
