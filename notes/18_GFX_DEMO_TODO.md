# 18_GFX_DEMO TODO

## Goal

Turn existing simulation state into stronger readable visuals, while keeping all effects presentation-only.

## Phase 1 — Effect Surface Audit

- [ ] Identify existing runtime state to drive:
  - reactor glow
  - engine thrust flames
  - EVA suit thrusters
  - welder sparks
  - repair/extraction progress
- [ ] Decide which effects are pure sprite overlays vs shader materials vs short-lived spawned particles.
- [ ] Confirm which systems already expose the required state without adding new authoritative simulation fields.

## Phase 2 — Reactor / Engine Overlays

- [ ] Add a reactor visual overlay child entity.
- [ ] Drive pulse intensity from reactor output / heat / reaction rate.
- [ ] Add engine flame overlays.
- [ ] Drive flame intensity from thrust demand and available engine power.
- [ ] Respect module orientation when placing the flame/glow art.

## Phase 3 — Player Suit Effects

- [ ] Add EVA back-thruster visual effect while accelerating in EVA.
- [ ] Make the effect directional relative to the actor’s movement/facing.
- [ ] Ensure standard/radiation/welder suits do not show the same EVA thrust effect unless intentionally configured.

## Phase 4 — Welding / Extraction Feedback

- [ ] Add spark effects while repair is in progress.
- [ ] Add spark effects while extraction is in progress.
- [ ] Add an in-world or screen-space progress bar for held repair/extraction work.
- [ ] Ensure these effects stop cleanly on cancel / completion / interruption.

## Phase 5 — Procedural Background Shader

- [ ] Add a shader-backed starfield / galaxy background.
- [ ] Generate stars and large-scale haze from seeded pseudorandom inputs.
- [ ] Support subtle parallax or depth feel if performance/readability allow.
- [ ] Replace the current flat-color encounter/sector background.

## Phase 6 — Authored Background Variation

- [ ] Add authored backdrop parameters to sector/encounter-facing data.
- [ ] Seed and tune at least:
  - calibration/test space
  - salvage field space
  - unstable/electrical space
  - hostile hold space
- [ ] Keep defaults safe when old save/config data is missing the new fields.

## Phase 7 — Polish And Safety

- [ ] Make sure effects do not leak entities on repeated encounter entry/exit.
- [ ] Make sure presentation-only timing does not affect rollback state.
- [ ] Tune colors/intensity so the game remains readable during damage, field, and HUD overlays.

## Test Plan

- [ ] Reactor glow responds to changing reactor state.
- [ ] Engine flames appear only while thrust is active.
- [ ] EVA thrusters appear only when an EVA-suited actor accelerates.
- [ ] Repair/extraction sparks and progress appear only while held work is active.
- [ ] Different nodes/sectors clearly show different background character.
- [ ] Entering/leaving encounters repeatedly does not duplicate or leak effect entities.
