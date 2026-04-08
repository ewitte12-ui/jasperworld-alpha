# Architecture Guide

A beginner-friendly walkthrough of how Jasper's World is organized. If you're new to Bevy or Rust game development, this is the place to start.

## How Bevy Works (Quick Primer)

Bevy is an Entity Component System (ECS) game engine. Three core concepts:

- **Entities** — Things in the game (the player, an enemy, a rock). Each entity is just an ID number.
- **Components** — Data attached to entities (position, health, speed). Components have no logic — they're just structs.
- **Systems** — Functions that run every frame and operate on entities that have certain components.

Example: A system might say "find every entity that has both `Transform` and `Enemy`, then move them along their patrol path." The engine finds matching entities automatically.

**Plugins** bundle related components and systems together. Each module in this project registers a plugin.

## Project Layout

```
src/
  main.rs              Entry point — creates the Bevy app and adds all plugins
  lib.rs               Re-exports the plugin registration
  constants.rs         Shared constants (tile size, etc.)
  states.rs            Game states (Title, Playing, Paused, etc.)
  window_geometry.rs   Window sizing and display setup

  animation/           Player and enemy animation (sprite flipping, model transforms)
  audio/               Sound effects and music playback
  collectibles/        Stars, health food — things Jasper picks up
  combat/              Damage, stomping enemies, tail slap attacks
  debug/               Debug overlays and developer tools
  dialogue/            In-game text/dialogue display
  enemies/             Enemy types, AI patrol behavior, spawning
  level/               Level loading, tile spawning, decorations, doors, sublevel switching
  lighting/            Directional and point light setup per level
  menu/                Main menu and UI screens
  particles/           Weather effects (leaves, rain, dust)
  physics/             Physics configuration (avian2d), collision layers, one-way platforms
  player/              Player controller, input handling, movement
  puzzle/              Gate/exit logic — "collect N stars to open the gate"
  rendering/           Camera, parallax scrolling, scene tinting, quad helpers
  resources/           Saved settings (audio, controls, graphics)
  save_load/           Save/load game state to disk
  tilemap/             Tile grid spawning, collider merging, autotile
  title/               Title screen scene (raccoon on a rock)
  ui/                  HUD (health bar, star count), pause menu
  vfx/                 Visual effects (vignette, screen overlays)

assets/                Runtime assets loaded by Bevy's asset server
  models/              3D models (.glb) — player, enemies, props, buildings, trees
  configs/             JSON configs for parallax backgrounds
  levels/              Compiled level data (tiles, entities, props, lights)

levels/                LDtk project file for visual level editing
tools/                 Build tools
  ldtk_compiler/       Compiles LDtk → compiled_levels.json
  backfill_ldtk.py     Populates LDtk from existing compiled data
```

## How a Level Loads

Here's what happens when you start a new game or enter a level, step by step:

1. **Game state changes** to `Playing` (see `states.rs`)
2. **`spawn_level_full`** in `level/mod.rs` runs:
   - Reads `assets/levels/compiled_levels.json` for tile grids, enemies, stars, doors, and props
   - Falls back to hardcoded Rust data if the JSON is missing
3. **Tiles are spawned** by `tilemap/spawn.rs`:
   - Each tile becomes a 3D model (GLB) with a collider
   - Adjacent tiles are merged into single wide colliders to prevent physics glitches
4. **Entities are spawned** from the JSON data:
   - Enemies get AI patrol behavior (`enemies/ai.rs`)
   - Stars and health food get collectible components
   - Doors get transition triggers
   - Props (rocks, flowers, trees) get decoration markers
5. **Background layers are spawned** by `rendering/parallax.rs`:
   - Reads from `assets/configs/forest_bg.json` (or city/subdivision)
   - Spawns mountains, trees, buildings, clouds at different Z depths
   - Each layer scrolls at a different speed (parallax effect)
6. **Lighting is configured** by `lighting/systems.rs`:
   - Each level has different light colors and directions
7. **The camera starts following** the player (`rendering/camera.rs`)

## How the Player Works

The player is built from several cooperating systems:

```
input.rs        → Reads keyboard/gamepad input, produces movement intent
controller.rs   → Applies physics-based movement using the intent
systems.rs      → Handles health, damage, respawning
animation/      → Chooses walk/idle/jump animation based on velocity
```

The player uses the **tnua** physics controller (a Bevy plugin) on top of **avian2d** (a 2D physics engine). This handles ground detection, jumping, and slopes automatically.

## How Parallax Scrolling Works

The background creates depth by scrolling layers at different speeds:

```
Z = +10   Foreground trees (in front of gameplay)
Z =   0   Tile plane (where Jasper runs)
Z = -38   Attenuation overlay (subtle darkening)
Z = -50   Near background (trees/houses/skyscrapers)
Z = -60   Clouds
Z = -70   Mountains (Forest only)
Z = -80   Far background (darker trees/distant buildings)
Z = -100  Sky
```

Each layer has a **parallax factor** (0.0 to 1.0). A factor of 0.0 means the layer doesn't move (feels close). A factor of 1.0 means it tracks the camera perfectly (feels infinitely far away, like the sky).

The system in `parallax.rs` reads the camera position each frame and positions each layer at `camera_x * factor`.

## How Levels Are Stored

Level data lives in two places:

### 1. Compiled Levels (`assets/levels/compiled_levels.json`)
Contains gameplay data per level per layer:
- **Tile grids** — 2D arrays of Solid/Platform/Empty
- **Enemies** — position, type, patrol range, health
- **Stars and health food** — position
- **Doors** — position, target layer
- **Props** — model path, position, scale, rotation
- **Lights** — position, color, intensity

### 2. Background Configs (`assets/configs/*.json`)
Contains parallax background data per level:
- **Mountains** — model, position, scale
- **Trees** — model list, spacing, scale range
- **Buildings/Houses** — model list, spacing, scale, rotation
- **Clouds** — texture, position, scale
- **Attenuation planes** — color, opacity, Z depth
- **Sky overlays** — color for overcast/night sky

Both are loaded at runtime. Edit the JSON to change the game without recompiling Rust.

## Key Concepts for Contributors

### Components as Markers
Many components are empty structs used as tags:
- `Decoration` — entity is despawned on level exit
- `TileEntity` — entity is despawned on sublevel switch
- `ForegroundDecoration` — entity is in front of or near gameplay
- `ParallaxBackground` — entity is a scrolling background layer

### The Spawn → Despawn Lifecycle
When you leave a level, everything tagged `Decoration` is destroyed. When you enter a sublevel door, everything tagged `TileEntity` is destroyed. This prevents entity leaks.

### Z Depth is Visual Only
The game is 2D — physics runs in the XY plane. The Z axis is only used for visual layering (what renders in front of what). A prop at Z=-15 is behind the player at Z=5.

### One-Way Platforms
Platforms use a special one-way collider (`physics/one_way.rs`). Jasper can jump through them from below but stand on top. This is handled by selectively enabling/disabling collision based on the player's vertical velocity.
