# TODO — Components Demo Implementation Breakdown

This file turns [05_COMPONENTS_DEMO.md](/home/adaml/code/lumenarch/notes/05_COMPONENTS_DEMO.md) into an implementation plan tied to the current codebase.

Priority remains:

1. establish real component-local manual control
2. unify manual control around deterministic command surfaces
3. prepare the runtime cleanly for [04_ARCH_DEMO.md](/home/adaml/code/lumenarch/notes/04_ARCH_DEMO.md)

## Guiding Constraints

* manual interaction must write staged command state rather than directly mutating outcomes
* component families should share panel patterns where sensible, but cockpit and turret can stay bespoke
* the default onboard view should become ship-local and more zoomed-in
* cockpit and turret views should intentionally break back out to world-relative control
* avoid building full final content breadth before proving the interaction grammar

## Expected Areas Of Change

Likely touched files and modules:

* `src/client/state.rs`
* `src/client/gameplay/components/`
* `src/client/gameplay/spawn/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/simulation/`

Likely new module areas:

* component command-state definitions
* station / focus state
* component panel rendering
* component-local control input handling

## Phase 1 — Station And Command Architecture

Goal:
Define the shared interaction model for component stations and the staged command surfaces they control.

Tasks:

- [ ] Add a station-focus model for the onboard player:
  - current station entity
  - current station family
  - current station interaction mode
- [ ] Define a shared command-state layer for manually writable component control.
- [ ] Separate component readable state from component writable command state.
- [ ] Ensure command state is consumed deterministically during fixed-step simulation.
- [ ] Decide which commands live:
  - per-module
  - per-vessel
  - per-station session

Likely files:

* `src/client/gameplay/components/`
* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/simulation/`

Definition of done:

* component-local manual controls can write staged command values without directly bypassing simulation

## Phase 2 — Ship-Local Interior Camera

Goal:
Make the default onboard experience ship-relative and distinct from flight view.

Tasks:

- [ ] Add ship-relative camera behavior for interior movement mode.
- [ ] Rotate the camera so the ship is always “up” on screen while onboard.
- [ ] Increase zoom-in for interior mode relative to cockpit / turret views.
- [ ] Preserve smooth transitions between:
  - ship-local interior
  - cockpit world view
  - turret station view
- [ ] Ensure shipboard marker, prompts, and local overlays remain readable in the rotated view.

Likely files:

* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/ui/`
* `src/client/state.rs`

Definition of done:

* walking the ship feels clearly ship-local, and leaving a station returns to that view

## Phase 3 — Station Entry / Exit Flow

Goal:
Make station use explicit rather than treating all nearby interaction as the same kind of action.

Tasks:

- [ ] Add station enter / exit interactions.
- [ ] Decide how the player enters a focused station mode:
  - `E`
  - click
  - both
- [ ] Allow leaving the station cleanly back to interior mode.
- [ ] Prevent conflicting movement / action input while in a focused station mode.
- [ ] Surface the active station family in the HUD.

Likely files:

* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/ui/`

Definition of done:

* the player can intentionally move from walking mode into a focused component-control mode and back

## Phase 4 — Cockpit Station

Goal:
Turn the cockpit into the canonical manual helm interface.

Tasks:

- [ ] Add cockpit station mode with world-view camera framing.
- [ ] Add helm controls for:
  - throttle demand
  - steering / turn demand
- [ ] Support both keyboard and mouse input paths.
- [ ] Add semi-diagetic cockpit readouts:
  - throttle position
  - helm steering rotation / demand
  - movement feedback
- [ ] Route cockpit input into shared ship command state rather than directly into motion behavior.

Likely files:

* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/components/`

Definition of done:

* the cockpit feels like piloting a helm station rather than just toggling flight mode

## Phase 5 — Turret Station

Goal:
Give turrets their own local control mode and readable aiming behavior.

Tasks:

- [ ] Add turret station mode using cockpit-style zoom but turret-hardpoint-local control.
- [ ] Add aim controls using:
  - mouse
  - keyboard
- [ ] Expose desired angle and actual angle separately.
- [ ] Add firing intent through the turret station.
- [ ] Show local turret cues:
  - actual vs desired angle
  - readiness / cooldown
  - local status if damaged or unstable
- [ ] Route these through turret command state rather than direct projectile spawning.

Likely files:

* `src/client/gameplay/components/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/simulation/combat.rs`

Definition of done:

* the player can man a turret as a distinct role with clear local feedback

## Phase 6 — Reactor Station And Reactor Simulation

Goal:
Replace the current abstract reactor “stabilize” interaction with a real manual control surface.

Tasks:

- [ ] Add reactor station mode.
- [ ] Add writable controls for:
  - reaction rate
  - turbine load
- [ ] Add readable reactor outputs:
  - internal heat
  - power output
  - fuel consumption
  - stability if modeled separately
- [ ] Update simulation so:
  - higher reaction rate increases heat and fuel consumption
  - higher turbine load reduces internal heat and increases power output
- [ ] Preserve failure / danger behavior under bad settings.

Likely files:

* `src/client/gameplay/components/`
* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/simulation/fields.rs`
* `src/client/gameplay/systems/simulation/mission.rs`
* `src/client/gameplay/systems/ui/`

Definition of done:

* reactor operation is a real balancing task, not only a generic repair-style action

## Phase 7 — Logistics Panels

Goal:
Make logistics stations into active operational panels rather than mostly passive status readouts.

Tasks:

- [ ] Add storage panel with:
  - contents breakdown
  - fill state
  - reserved / usable distinction if needed
- [ ] Add manipulator panel with:
  - source / target
  - held load or current job
  - manual transfer or routing controls
- [ ] Add processor panel with:
  - recipe list
  - current recipe
  - progress
  - input / output state
- [ ] Route logistics controls into staged command state.

Likely files:

* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/simulation/logistics.rs`

Definition of done:

* logistics modules feel operable rather than merely inspectable

## Phase 8 — Shared Panel Framework

Goal:
Avoid rewriting bespoke UI plumbing for every future component subtype.

Tasks:

- [ ] Define a common panel shell:
  - title
  - health / condition
  - warnings
  - local controls area
- [ ] Define family-level panel groupings for:
  - cockpit
  - turret
  - reactor family
  - logistics family
- [ ] Separate family-specific content from generic panel layout code.
- [ ] Leave clear extension points for:
  - shields
  - detectors
  - drone stations
  - memory banks

Likely files:

* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/spawn/scene/`

Definition of done:

* new component families can plug into the station UI model without a fresh architecture pass

## Phase 9 — Future-Family Scaffolding

Goal:
Ensure the interaction model can support upcoming component types without redesign.

Tasks:

- [ ] Define placeholder command/read models for:
  - shields
  - detectors
  - drone stations
  - memory banks
- [ ] For detectors, allow initial information-layer toggles to piggyback on the current debug overlay where practical.
- [ ] For shields, sketch directional-control-ready command fields even if no shield component is yet playable.
- [ ] For drone stations, define task-selection command semantics.
- [ ] For memory banks, define readable stored-data presentation semantics.

Likely files:

* `src/client/gameplay/components/`
* `src/client/gameplay/systems/ui/`
* `src/client/state.rs`

Definition of done:

* the component-interaction model is visibly ready for the next families even if their content is still partial

## Phase 10 — Correct Earlier-Slice Shortcuts

Goal:
Replace abstractions from earlier milestones that would conflict with the component-demo direction.

Tasks:

- [ ] Replace generic cockpit “return to flight” behavior with cockpit station flow.
- [ ] Replace global-only turret fire assumptions with turret-local command support.
- [ ] Replace reactor “stabilize only” interaction with the real control panel.
- [ ] Upgrade logistics inspection from passive display to control-capable panel flow.
- [ ] Ensure the ship-local onboard view is now the default non-station interior mode.

Likely files:

* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/ui/`

Definition of done:

* the runtime no longer depends on placeholder interaction models where this slice has introduced real ones

## Phase 11 — ARCH Compatibility Pass

Goal:
Make sure 04 can build directly on this work rather than creating a second control architecture.

Tasks:

- [ ] Audit every manual station control and identify its future writable register equivalent.
- [ ] Document the mapping from:
  - station UI control
  - staged command state
  - future ARCH writable register
- [ ] Ensure no critical manual path still bypasses staged command state.
- [ ] Add naming that will remain coherent once ARCH writes those controls too.

Likely files:

* `src/client/gameplay/components/`
* `src/client/gameplay/systems/simulation/`
* notes only if needed

Definition of done:

* 04 ARCH can target the established command surfaces directly

## Phase 12 — Playtest And Tuning

Goal:
Tune the new interaction grammar until stations feel distinct, useful, and readable.

Tasks:

- [ ] Playtest interior view orientation and zoom.
- [ ] Tune cockpit vs turret camera transitions.
- [ ] Tune reactor response so the control tradeoff is understandable but dangerous.
- [ ] Tune logistics panels so they are useful without becoming spreadsheet-heavy.
- [ ] Check whether the player naturally understands when to walk, when to station, and when to leave.

Definition of done:

* players can clearly feel the difference between walking the ship, piloting it, manning a turret, managing a reactor, and operating logistics

## Immediate Next Task

Start with **Phase 1**:

* define station-focus state
* define per-component command-state structures
* route manual interaction toward staged commands instead of direct effect mutation

That is the foundation the rest of the slice, and later 04 ARCH work, depends on.
