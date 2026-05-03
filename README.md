# LUMEN//ARCH

LUMEN//ARCH is a top-down 2D sci-fi systems game about building a patchwork ship, surviving inside it, and gradually moving from direct hands-on control to guided autonomy.

The project’s tone comes from a galaxy full of still-functioning dead machinery: abandoned stations refining forever, defense systems without masters, and drifting vessels trapped in obsolete routines. The player pilots a modular "Knot" through that graveyard, scavenging, fighting, repairing, and eventually programming more of the ship than they manually operate.

At the center of the design is a single progression arc:

> direct control -> partial automation -> system architecture

`ARCH` is the precise, deterministic control layer. `LUMEN` is the optimization layer that nudges system behavior rather than issuing direct commands. Around those systems sits a playable ship sim with crew embodiment, environmental hazards, ship editing, sector travel, and rollback multiplayer.

## Current Slice

The repo currently supports a substantial playable slice including:

- lobby/bootstrap into multiplayer rollback sessions
- docked ship management and player refit
- sector-map travel and encounter launch
- shared-crew encounters on a single ship
- hostile ships, salvage, repair, extraction, suits, and cargo
- in-editor ship construction and component configuration
- ARCH/LUMEN authoring surfaces and runtime computer behavior

## Running

Interactive client:

```bash
cargo run
```

Headless mode:

```bash
cargo run -- --headless
```

Tests:

```bash
cargo test -q
```

The headless mode is intended for simulation-oriented testing and future automated validation. It boots the game without the normal UI/render stack and is used by the crate’s simulation tests.

## Repository Structure

The codebase is organized mostly around how the game is played rather than around generic engine layers.

- [src/main.rs](src/main.rs)  
  App construction, plugin setup, rollback registration, runtime mode selection, and schedule/system registration.

- [src/gameplay](src/gameplay)  
  Runtime encounter logic.
  - `components/`: ECS state for players, ships, modules, projectiles, logistics, missions, and simulation data
  - `helpers/`: fixed-point math, collision math, interaction helpers, ship helpers, status helpers
  - `spawn/`: runtime scene, ship, module, salvage, and HUD spawning
  - `systems/`: authoritative gameplay systems and presentation systems

- [src/editor](src/editor)  
  Ship refit/editor flow, component configuration, program editing, and editor UI.

- [src/netcode](src/netcode)  
  Lobby/bootstrap, GGRS input encoding/decoding, rollback state, presentation sync, and session lifecycle.

- [src/ship](src/ship)  
  Data model for ship definitions, module specs, ARCH programs, LUMEN programs, storage, and enemy ship definitions.

- [src/state](src/state)  
  Frontend/UI resources and high-level presentation state.

- [src/lobby.rs](src/lobby.rs)  
  Lobby UI and local profile editing.

- [src/docked.rs](src/docked.rs)  
  Docked-state presentation and campaign persistence hooks.

- [src/sector_map](src/sector_map)  
  Sector-map UI, input, and layout.

- [docs/src](docs/src)  
  Design and architecture notes, including the project concept.

- [notes](notes)  
  Vertical-slice plans and TODO trackers used to drive implementation.

- [saves](saves)  
  Editable runtime data such as balance config, sector layout, campaign state, and saved ship/enemy definitions.

- [assets](assets)  
  Fonts, tiles, actor sprites, and other game assets.

## Technical Architecture

### ECS-first runtime

The project is built on Bevy 0.18 and treats most game behavior as ECS systems operating over explicit component state. The gameplay code is split into:

- deterministic simulation systems
- presentation/UI systems
- spawn/teardown systems
- editor and frontend systems

That split matters because the game is also built around rollback networking.

### Rollback and networking model

Multiplayer is built with `bevy_ggrs` / `ggrs`.

Important rules:

- rollback owns authoritative in-session state
- player input is the only intended source of nondeterminism
- deterministic gameplay runs in `GgrsSchedule`
- presentation mirrors rollback state but should not author it

The game uses a pre-session lobby/bootstrap layer to assemble the player list and session config, then starts a rollback session for actual play. The current model is:

- one shared player ship
- multiple synchronized crew actors
- host-authored meta transitions for docked / sector map / player editor
- shared deterministic encounter simulation once in-session

The authoritative in-session state is centered on `RollbackGameState`, including phase, ship snapshot, progression, sector state, and mission report.

### APP runtime modes

There are currently two runtime modes in [src/main.rs](src/main.rs):

- `Interactive`
- `Headless`

Interactive mode loads the normal Bevy UI/window stack. Headless mode builds a smaller plugin set so simulation tests can exercise rollback and gameplay logic without requiring a desktop render loop.

### World space vs ship space

One of the game’s recurring implementation concerns is the difference between:

- world space: ship movement, projectiles, hostile ships, camera positioning
- ship/local space: crew movement inside ships, module positions, atmosphere tiles, field sampling, station usage

The player frequently transitions between interior movement, EVA movement, and focused station control. A lot of gameplay code exists to map correctly between those spaces:

- crew have local ship-relative positions when aboard
- ships have world transforms and fixed-point simulation state
- cameras and overlays interpret that differently depending on whether the player is free-moving, EVA, in cockpit control, or on a turret/station

This is one of the most important cross-cutting ideas in the repo. Bugs often happen when the wrong space is treated as authoritative.

## Cross-Cutting Concerns

### 1. Balance config is data, not code

Gameplay tuning is intentionally moving out of hardcoded values and into [saves/balance_config.json](saves/balance_config.json), loaded by [src/balance.rs](src/balance.rs).

That config covers things like:

- ship movement
- reactor and power behavior
- field effects
- oxygen and decompression behavior
- combat tuning
- player movement and suit modifiers
- interaction durations
- mission pacing

If something feels like a tuning knob rather than a structural invariant, it probably belongs in the balance config.

### 2. Determinism matters

The project uses fixed-point math heavily (`fixed`, `cordic`) because the simulation is expected to converge identically across peers and in rollback re-simulation.

When touching gameplay code:

- avoid introducing floating-point math into authoritative simulation paths
- avoid hidden randomness
- avoid local-only side effects in rollback-owned systems
- keep input as the only intended nondeterministic source

When a formula gets complex, the preferred pattern is to push it into helper functions and clamp/assert intermediate values, especially in collision and simulation code.

### 3. UI should capture intent, not own gameplay state

A repeated architectural rule in the repo is:

- UI/presentation systems gather local intent
- rollback/gameplay systems apply authoritative state changes

This matters particularly for:

- station controls
- sector-map travel
- editor actions that affect synchronized state
- any system that used to mutate gameplay state directly from Bevy UI

### 4. Simulation testing is part of the architecture

The project now includes headless simulation testing in [src/sim_tests.rs](src/sim_tests.rs). These tests are meant to exercise real flows, not just tiny isolated functions.

The current simulation test covers:

- host lobby bootstrap
- editor open/close
- sector map transition
- node selection
- encounter launch
- cockpit use from spawn

The long-term intent is to grow this into broader regression coverage for rollback, simulation stability, and scenario scripting.

## Data and Content

Several important game data sets are editable without recompiling:

- [saves/balance_config.json](saves/balance_config.json)
- [saves/sector_layout.json](saves/sector_layout.json)
- [saves/campaign_state.json](saves/campaign_state.json)
- [saves/player_ship.json](saves/player_ship.json)
- [saves/enemy_ships.json](saves/enemy_ships.json)

That makes the repo a mix of engine code, game code, and live game data.

## Notes for Contributors

- Read [docs/src/CONCEPT.md](docs/src/CONCEPT.md) first for tone and design intent.
- Check [notes](notes) for the current vertical-slice implementation plans.
- If you are changing gameplay feel, check whether the value should live in balance config instead of code.
- If you are changing synchronized behavior, ask whether the change belongs in rollback state/input rather than presentation state.
- If you are adding complex math in simulation, prefer fixed-point helpers over ad hoc inline calculations.
- If you are adding tests, prefer realistic headless simulation flows when possible.

## Project Status

This is an active prototype with a lot of implemented systems and some deliberate rough edges:

- some legacy/editor paths still exist while newer flows supersede them
- not every slice is fully generalized yet
- the codebase prioritizes vertical-slice momentum and simulation integrity over polish

Even so, the current structure is deliberate: the project is already organized around the long-term idea of a deterministic modular ship sim with embodied crew, salvage-driven progression, and increasingly expressive automation.
