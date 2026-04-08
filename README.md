# Jasper's World

A 2D platformer starring Jasper the raccoon, built with [Bevy](https://bevyengine.org/) (Rust).

## About

Jasper's World is a side-scrolling platformer rendered with a 2.5D camera system — 2D gameplay on 3D-lit scenes. Guide Jasper through three distinct levels, collecting stars, solving puzzles, and stomping enemies along the way.

### Levels

- **Forest** — Grassy meadows with trees, mountains, and falling leaves. Jasper's home turf.
- **Subdivision** — Rainy suburban neighborhood with houses, brick platforms, and overcast skies.
- **City** — Nighttime urban environment with tall skyscrapers, scaffolding platforms, moonlight, and drifting dust particles.

Each level has three explorable layers accessed through doors: a main surface layer, an underground sublevel (cave, sewer, or subway), and a rooftop layer.

### Features

- 3D-lit 2D platformer using Bevy's Camera3d with orthographic projection
- Three levels, each with three explorable layers
- Parallax scrolling backgrounds with weather effects (leaves, rain, dust)
- Enemies with patrol AI and stomp-based combat
- Star collection and gate puzzles
- Save/load system with multiple slots
- Controller and keyboard support with rebindable controls
- Custom 3D models (Tripo AI-generated) for characters, props, and backgrounds
- Data-driven level design — positions and layouts stored in JSON, not hardcoded

## Building & Running

Requires [Rust](https://www.rust-lang.org/tools/install) (2024 edition).

```bash
cargo run            # Run in debug mode
cargo run --release  # Run in release mode (recommended for gameplay)
cargo test           # Run tests
cargo clippy         # Lint
```

## Project Structure

For a detailed walkthrough of the codebase, see:

- [ARCHITECTURE.md](ARCHITECTURE.md) — How the code is organized, what each module does, and how systems connect
- [ASSETS.md](ASSETS.md) — Asset pipeline, model sources, JSON configs, and how to add new content

## Level Editor

Levels can be edited visually using [LDtk](https://ldtk.io/) (a free 2D level editor). The project file is at `levels/jasperworld.ldtk`.

**Workflow:**
1. Open `levels/jasperworld.ldtk` in LDtk
2. Edit tiles, place entities (enemies, stars, doors, props, lights)
3. Save in LDtk
4. Compile to game format: `cargo run -p ldtk_compiler -- --input levels/jasperworld.ldtk --output assets/levels/compiled_levels.json`
5. Run the game — it reads from `compiled_levels.json` automatically

Background visuals (parallax mountains, trees, buildings) are configured separately in `assets/configs/*.json` files. See [ASSETS.md](ASSETS.md) for details.

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Support

Jasper's World is open-source under the MIT License. You are free to use, modify, and share the code.

If you enjoy the game and want to support development:

- [GitHub Sponsors](https://github.com/sponsors/ewitte12-ui)
- [Ko-fi](https://ko-fi.com/ewitte12ui)
- [Itch.io](https://itch.io/profile/ericjwi) — Coming soon

Your support is greatly appreciated and helps fund new features, art, and updates!

### Notes
- Attribution: Please keep the MIT license and copyright notice.
- Commercial use: Allowed by the MIT license.
- Issues and PRs: Contributions are welcome.
