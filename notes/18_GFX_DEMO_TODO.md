# 18_GFX_DEMO TODO

## Goal

Turn existing simulation state into stronger readable visuals, while keeping all effects presentation-only.

## Phase 1 — Effect Surface Audit

- [x] Identify existing runtime state to drive:
  - reactor glow
  - engine thrust flames
  - EVA suit thrusters
  - welder sparks
  - repair/extraction progress
- [x] Decide which effects are pure sprite overlays vs shader materials vs short-lived spawned particles.
- [x] Confirm which systems already expose the required state without adding new authoritative simulation fields.

## Phase 2 — Reactor / Engine Overlays

- [x] Add a reactor visual overlay child entity.
- [x] Drive pulse intensity from reactor output / heat / reaction rate.
- [x] Add engine flame overlays.
- [x] Drive flame intensity from thrust demand and available engine power.
- [x] Respect module orientation when placing the flame/glow art.

## Phase 3 — Player Suit Effects

- [x] Add EVA back-thruster visual effect while accelerating in EVA.
- [x] Make the effect directional relative to the actor’s movement/facing.
- [x] Ensure standard/radiation/welder suits do not show the same EVA thrust effect unless intentionally configured.

## Phase 4 — Welding / Extraction Feedback

- [x] Add spark effects while repair is in progress.
- [x] Add spark effects while extraction is in progress.
- [x] Add an in-world or screen-space progress bar for held repair/extraction work.
- [x] Ensure these effects stop cleanly on cancel / completion / interruption.

## Phase 5 — Procedural Background Shader

- [x] Add a shader-backed starfield / galaxy background.
- [x] Generate stars and large-scale haze from seeded pseudorandom inputs.
- [x] Support subtle parallax or depth feel if performance/readability allow.
- [x] Replace the current flat-color encounter/sector background.

## Phase 6 — Authored Background Variation

- [x] Add authored backdrop parameters to sector/encounter-facing data.
- [x] Seed and tune at least:
  - calibration/test space
  - salvage field space
  - unstable/electrical space
  - hostile hold space
- [x] Keep defaults safe when old save/config data is missing the new fields.

## Phase 7 — Polish And Safety

- [x] Make sure effects do not leak entities on repeated encounter entry/exit.
- [x] Make sure presentation-only timing does not affect rollback state.
- [x] Tune colors/intensity so the game remains readable during damage, field, and HUD overlays.

## Test Plan

- [x] Reactor glow responds to changing reactor state.
- [x] Engine flames appear only while thrust is active.
- [x] EVA thrusters appear only when an EVA-suited actor accelerates.
- [x] Repair/extraction sparks and progress appear only while held work is active.
- [x] Different nodes/sectors clearly show different background character.
- [x] Entering/leaving encounters repeatedly does not duplicate or leak effect entities.
