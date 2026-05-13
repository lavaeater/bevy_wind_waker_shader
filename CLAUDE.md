# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

`bevy_wind_waker_shader` is a Bevy plugin crate that provides a toon shader mimicking the character shading from The Legend of Zelda: The Wind Waker. It is a library crate (no `main.rs`) published to crates.io.

## Commands

```bash
# Check for compile errors
cargo check

# Run all tests including doc tests
cargo test --all-features
cargo test --doc

# Lint (CI treats warnings as errors)
RUSTFLAGS="-D warnings" cargo clippy --tests --examples

# Format check
cargo fmt --check --all

# Build docs
RUSTDOCFLAGS="--deny warnings" cargo doc --no-deps --all-features

# Run an example
cargo run --example basic
cargo run --example scene
cargo run --example change_color_palette
```

## Architecture

The crate has these source files and two embedded assets:

- **`src/lib.rs`** — re-exports public API and defines the `prelude` module
- **`src/components.rs`** — core types for the Wind Waker shader: `WindWakerShader` (the `MaterialExtension`), `WindWakerShaderBuilder` (builder with hard-coded hex color palettes for all 12 time-of-day × weather combinations), `TimeOfDay`, `Weather`, and `ExtendedMaterial` type alias.
- **`src/plugin.rs`** — `WindWakerShaderPlugin`: loads the WGSL shader and ZAtoon mask texture as internal (embedded) assets at fixed UUID handles, registers the `MaterialPlugin`, and adds the two material-customization systems.
- **`src/systems.rs`** — two `PreUpdate` systems for Wind Waker: `customize_scene_materials` (handles `SceneRoot` entities, waits for the scene to be ready) and `customize_standard_materials` (handles bare mesh entities).
- **`src/flat.rs`** — `FlatShader` + `FlatShaderBuilder` + `FlatShaderPlugin` + systems. Sable-inspired flat toon shader that *preserves* the mesh's original texture colors (unlike Wind Waker which replaces them). Supports game-wide automatic application via `FlatShaderPlugin::global()`.

### Material extension pattern

The shader extends Bevy's PBR pipeline via `MaterialExtension`. Users insert a `WindWakerShader` component (built via `WindWakerShaderBuilder`) onto their entity. The plugin's systems detect this, clone the existing `StandardMaterial`, wrap it in `ExtendedMaterial`, and replace the material handle — removing the `WindWakerShader` component when done to avoid re-processing.

### Embedded assets

Both `src/assets/toon_shader.wgsl` (WGSL fragment shader) and `src/assets/ZAtoon.png` (1D toon mask texture) are embedded in the binary via `load_internal_asset!` / `include_bytes!` and referenced by stable UUID handles defined in `components.rs`. No runtime asset loading is required.

### Bevy 0.18 observer API

Observers use `On<E, B>` (not `Trigger`) as the first param, `Add` (not `OnAdd`) for component-add hooks, and `trigger.event_target()` (not `trigger.target()`) to get the entity.

### WGSL shader overview

The Wind Waker fragment shader (`toon_shader.wgsl`):
1. Computes standard PBR lighting to get a raw intensity.
2. Samples the ZAtoon 1D mask texture using the red channel as U coordinate to quantize lighting into toon bands.
3. Mixes `shadow_color` and `highlight_color` based on the quantized result.
4. Adds a rim highlight (quartic falloff) using `rim_color`.
5. Multiplies by the original base texture color, then applies Bevy's post-lighting processing (fog, tonemapping).

The flat shader (`flat_shader.wgsl`) is identical up to step 3, but in step 4 it mixes `shadow_tint`/`highlight_tint` multipliers and applies them *onto* the texture color rather than replacing it. No rim highlight.

## Compatibility

This crate tracks Bevy versions 1:1. The current version targets **Bevy 0.18** and uses the Bevy 2024 edition. When upgrading Bevy versions, check `MaterialExtension`, `ExtendedMaterial`, `load_internal_asset!`, and `uuid_handle!` APIs — these have changed across releases.
