# Jasper's World

A 2D platformer starring Jasper the raccoon, built with [Bevy](https://bevyengine.org/) (Rust).

## About

Jasper's World is a side-scrolling platformer rendered with a 2.5D camera system — 2D gameplay on 3D-lit scenes. Guide Jasper through three distinct levels, collecting stars, solving puzzles, and stomping enemies along the way.

### Levels

- **Forest** — Grassy meadows with trees, mountains, and falling leaves. Jasper's home turf.
- **Subdivision** — Rainy suburban neighborhood with houses, brick platforms, and overcast skies.
- **City** — Nighttime urban environment with tall skyscrapers, scaffolding platforms, moonlight, and drifting dust particles.

### Features

- 3D-lit 2D platformer using Bevy's Camera3d with orthographic projection
- Three levels, each with three explorable layers (accessed through doors)
- Parallax scrolling backgrounds with weather effects (leaves, rain, dust)
- Enemies with patrol AI and stomp-based combat
- Star collection and gate puzzles
- Save/load system with multiple slots
- Controller and keyboard support with rebindable controls

## Building & Running

Requires [Rust](https://www.rust-lang.org/tools/install) (2024 edition).

```bash
cargo run            # Run in debug mode
cargo run --release  # Run in release mode (recommended for gameplay)
cargo test           # Run tests
cargo clippy         # Lint
```

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Support & Donations

Jasper's World is open-source under the MIT License. You are free to use, modify, and share the code.

If you enjoy the game and want to support development:

- GitHub Sponsors / Ko-fi / Patreon: Your donations help keep the project alive and growing.
  - GitHub Sponsors: https://github.com/sponsors/ewitte12-ui (update if different)
  - Ko-fi: https://ko-fi.com/yourname - coming soon
  - Patreon: https://patreon.com/yourname - comming soon
- Itch.io Build: COMMING SOON - Get a fully compiled version on itch.io for $2 (or pay what you want). Payment is optional -- the source code remains free.
  - Add your itch.io page link here: https://itch.io/profile/ericjwi

Your support is greatly appreciated and helps fund new features, art, and updates!

### Notes
- Attribution: Please keep the MIT license and copyright notice.
- Commercial use: Allowed by the MIT license.
- Issues and PRs: Contributions are welcome.
