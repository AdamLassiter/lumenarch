# 20_AUTOMATION_DEMO TODO

## Goal

Implement a first coherent family of automation-support sensor and monitor modules that expand what ARCH can perceive and react to.

## Phase 1 — Sensor Family Spec

- [x] Define the new component families and variants:
  - life sign detectors
  - ship detectors
  - ship damage detectors
  - power state monitor
  - heat / hazard monitor
  - logistics demand beacon
- [x] Decide whether these are separate `ModuleKind`s, variants within a shared sensor family, or a mixed strategy.
- [x] Define first-pass art/editor naming so the family reads clearly in the toolbox.
- [x] Decide which components are player-available early vs mid vs late.

## Phase 2 — Detection Semantics

- [x] Define what counts as:
  - friendly life sign
  - hostile life sign
  - nearby ship
  - hostile ship
  - incoming ship damage
  - critical damage state
- [x] Define first-pass sensor range semantics.
- [x] Decide whether ranges are:
  - module-centered
  - ship-centered
  - variant-dependent
- [x] Define first-pass directional encoding.
- [x] Define first-pass distance/intensity quantization for higher tiers.

## Phase 3 — Register Surface Design

- [x] Define canonical detector register names.
- [x] Decide which outputs are present for low / mid / high variants.
- [x] Keep the register surface compact enough for simple ARCH use.
- [x] Reuse existing naming patterns where possible instead of inventing a parallel vocabulary.
- [x] Decide whether any monitors expose summary-only outputs versus both summary and detail outputs.

## Phase 4 — Runtime Simulation Hooks

- [x] Add runtime module state / helper logic for life sign detection.
- [x] Add runtime module state / helper logic for ship detection.
- [x] Add runtime module state / helper logic for ship damage detection.
- [x] Add runtime summary logic for power stress monitoring.
- [x] Add runtime summary logic for heat/hazard monitoring.
- [x] Add runtime summary logic for logistics demand monitoring.
- [x] Keep all outputs deterministic and safe for rollback.

## Phase 5 — ARCH Integration

- [x] Expose detector outputs through component/channel register surfaces.
- [x] Ensure detector modules are visible and legible in the in-mission component UI.
- [ ] Add or update sample ARCH templates that use the new modules, such as:
  - combat posture
  - boarding alert
  - power shedding
  - logistics escalation
- [x] Verify that simple scripts can use the low-tier detectors without excessive polling complexity.

## Phase 6 — Editor / Balance / Progression

- [x] Add the new parts to the editor toolbox.
- [x] Define costs, integrity, and power draw by tier.
- [x] Gate stronger detectors appropriately in progression.
- [x] Make sure enemy/debug editor can use them without artificial availability issues.
- [x] Keep the sensor family visually and textually grouped so it does not become toolbox noise.

## Phase 7 — UI / Readability

- [x] Add readable module console text for detector outputs.
- [x] Make direction/distance outputs understandable without requiring raw-number memorization.
- [x] Add status/help text or examples where needed so players understand why these modules matter.
- [x] Make sure the player can distinguish:
  - binary detector tier
  - directional tier
  - survey tier

## Phase 8 — Scenario / Demo Validation

- [x] Add or identify encounter scenarios where the new detector modules matter.
- [x] Verify boarding-related detector usefulness.
- [x] Verify ship-contact detector usefulness in combat posture automation.
- [x] Verify damage detectors meaningfully support shield/maneuver/retreat logic.
- [x] Verify power and logistics monitors help tie together existing component families.

## Test Plan

- [x] A low-tier life sign detector can trigger a simple boarding alarm script.
- [x] A mid/high-tier ship detector can support direction-aware combat automation.
- [x] A damage detector can distinguish "taking damage now" from merely "some module is damaged."
- [x] Monitor components do not produce nondeterministic outputs across repeated runs.
- [x] Detector outputs update correctly on repeated encounter entry/exit.
- [x] Sensor-heavy ships feel more automatable than sensor-poor ships without removing the need for thoughtful scripting.

## Test Questions

- [ ] Do these modules make ARCH feel more like ship engineering and less like blind scripting?
- [ ] Do the tiers create meaningful information upgrades instead of only bigger numbers?
- [ ] Does sensor placement matter enough to influence ship design?
- [ ] Do the monitor components help connect existing systems without flattening them into one generic "AI core"?
