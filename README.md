# Procedural Generation

A 3D procedural generation demo built with Bevy game engine, showcasing wave function collapse algorithms for creating townscapes and architectural structures.

## Features

- Procedural terrain and structure generation using `bevy_ghx_proc_gen`
- Interactive 3D visualization with pan-orbit camera controls
- Wave function collapse implementation for generating coherent layouts
- FPS counter and debug tooling
- Physics integration with Avian3D

## Built With

- [Bevy 0.13](https://bevyengine.org/) - Game engine
- [bevy_ghx_proc_gen](https://crates.io/crates/bevy_ghx_proc_gen) - Procedural generation library
- [bevy_panorbit_camera](https://crates.io/crates/bevy_panorbit_camera) - Camera controls
- [Avian3D](https://crates.io/crates/avian3d) - Physics engine

## Running

```bash
cargo run --release
```

Development builds include basic optimization for faster iteration while keeping dependencies fully optimized.

## Project Structure

- `src/pillars.rs` - Main generation logic for architectural elements
- `src/townscape.rs` - Townscape generation implementation
- `src/plugin.rs` - Core procedural generation plugin
- `src/rules.rs` - Wave function collapse rules
- `src/fps.rs` - FPS monitoring
