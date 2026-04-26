# TODO — Systems Demo Implementation Breakdown

## Purpose

This file turns `SYSTEMS_DEMO.md` into a concrete implementation checklist for the current codebase.

The focus is:

* move from ship-scale combat into ship-interior systems gameplay
* keep scope tight around one new inner-ship loop
* sequence work so each step unlocks visible progress
* avoid prematurely implementing full ARCH, LUMEN, logistics, drones, or broad field simulation

---

## Current Baseline

Already in place from the first playable demo:

* shared editor/runtime ship pipeline
* editor-to-runtime launch path
* runtime ship spawning from edited ship data
* aggregate ship movement and power model
* player weapons and projectiles
* hostile encounter flow
* module integrity / destruction
* salvage reward loop
* return-to-editor flow
* minimal progression currency
* editor and runtime HUD scaffolding

Not yet in place for this milestone:

* player embodiment inside the ship
* manual module interaction
* degraded module states beyond destruction
* field-based onboard hazards
* repair and stabilization actions
* first playable ARCH interaction
* ship-interior readability and status tooling

---

## Phase 1 — Player-On-Ship Representation

### Goal

Represent the player as an entity or controllable presence within the ship during runtime, rather than only treating the player as the ship itself.

### Tasks

- [ ] Audit the current gameplay module structure and decide where the internal player representation should live.
- [ ] Add a dedicated internal player component/resource, likely something like:
  - [ ] `ShipboardPlayer`
  - [ ] `PlayerShipAssignment`
  - [ ] `InternalPosition`
  - [ ] `CurrentStation`
- [ ] Decide on the first movement model:
  - [ ] tile-to-tile movement
  - [ ] node-based movement between module anchors
  - [ ] free local movement within ship bounds
- [ ] Add a runtime concept of walkable or reachable interior spaces.
- [ ] Define how the player position is derived from ship layout:
  - [ ] hull / corridor tiles
  - [ ] adjacency graph
  - [ ] module interaction anchor points
- [ ] Add a mode switch between:
  - [ ] ship control
  - [ ] internal control
- [ ] Add a simple visual or marker for current player location inside the ship.
- [ ] Ensure internal player state persists cleanly during a mission and resets cleanly on mission return.

### Files Likely Affected

* gameplay runtime state module
* gameplay input module
* ship spawn/runtime module
* HUD/UI module

### First-Pass Simplifications

- [ ] Do not implement full collision-rich character movement.
- [ ] It is acceptable for the first pass to snap between tiles or interaction nodes.
- [ ] It is acceptable for the player to be constrained to a simplified “interior overlay” representation rather than literal room geometry.

### Phase 1 Definition Of Done

* the player can switch into an internal-control mode during a mission
* the player has a position aboard the ship that is not merely “at the cockpit”
* moving to different locations aboard the ship is possible and readable

---

## Phase 2 — Interaction Framework

### Goal

Allow the player to interact with nearby ship systems in a generic and extensible way.

### Tasks

- [ ] Add an `Interactable` marker or equivalent component for runtime modules that support shipboard actions.
- [ ] Add interaction range / adjacency detection from the internal player position.
- [ ] Add a generic interaction prompt pipeline:
  - [ ] current nearby target
  - [ ] prompt text
  - [ ] unavailable / blocked reason if needed
- [ ] Add a generic interaction event or command flow, likely something like:
  - [ ] `InteractWithModule`
  - [ ] `BeginHeldInteraction`
  - [ ] `CompleteHeldInteraction`
- [ ] Support at least two interaction forms:
  - [ ] instant interaction
  - [ ] hold-to-interact
- [ ] Add module-type-specific dispatch for:
  - [ ] cockpit
  - [ ] reactor
  - [ ] turret
  - [ ] shield emitter if present
  - [ ] damaged module / repair target
- [ ] Ensure interactions are cancelled cleanly if:
  - [ ] the player moves away
  - [ ] the module is destroyed
  - [ ] the mission ends

### Files Likely Affected

* gameplay interaction module
* runtime module components
* HUD/prompt UI

### First-Pass Simplifications

- [ ] Only one interaction target may be active at once.
- [ ] Prioritize nearest interactable rather than building a complex selection system.
- [ ] Use generic hold durations before tuning per-component timings.

### Phase 2 Definition Of Done

* being near a supported module shows an interaction prompt
* pressing or holding interact performs a component-specific action
* the interaction system is generic enough to support more module actions later

---

## Phase 3 — Expanded Runtime Module State

### Goal

Make modules behave like systems under stress rather than binary alive/dead tiles.

### Tasks

- [ ] Audit current runtime module data and decide where additional live state should live.
- [ ] Add per-module state fields, likely including:
  - [ ] `current_heat`
  - [ ] `electrical_instability`
  - [ ] `is_disabled`
  - [ ] `needs_attention`
  - [ ] `last_interaction_time` if needed
- [ ] Decide whether state is attached directly to module entities or mirrored in a ship-level cache.
- [ ] Define degraded behavior for first supported module types:
  - [ ] reactor
  - [ ] engine
  - [ ] turret
  - [ ] battery
  - [ ] shield if present
- [ ] Add systems that recalculate ship capability from both:
  - [ ] installed modules
  - [ ] live module state
- [ ] Ensure degraded modules remain visible and inspectable.
- [ ] Distinguish:
  - [ ] healthy
  - [ ] degraded
  - [ ] disabled
  - [ ] destroyed

### Files Likely Affected

* runtime module components
* movement/power/combat systems
* inspection/status UI

### First-Pass Simplifications

- [ ] Avoid building full register-level simulation here.
- [ ] Use a small number of well-defined degraded states.
- [ ] It is acceptable to support only a subset of installed module kinds at first.

### Phase 3 Definition Of Done

* modules can remain present but impaired
* ship behavior changes when modules overheat or become unstable
* not all failures are full destruction

---

## Phase 4 — Basic Field Simulation

### Goal

Introduce a narrow and readable field layer that creates local onboard pressure.

### Tasks

- [ ] Decide how the first field implementation will be represented in code:
  - [ ] ad-hoc per-module overlap checks
  - [ ] shared field emitter component
  - [ ] ship-local sampled values
- [ ] Add first-pass field emitter definitions for:
  - [ ] heat
  - [ ] cooling
  - [ ] electrical interference
- [ ] Define which module types emit which first-pass fields:
  - [ ] reactor emits heat
  - [ ] radiator or cooler emits negative heat if present
  - [ ] damaged systems emit electrical instability
  - [ ] engines and turrets optionally emit heat under use
- [ ] Add sampling logic for:
  - [ ] player position
  - [ ] module positions
- [ ] Add threshold-based effects for first-pass interpretation:
  - [ ] player heat danger
  - [ ] module heat damage or degradation
  - [ ] module electrical unreliability or disable chance
- [ ] Decide how these values update relative to existing runtime systems.
- [ ] Add debug tooling for field visualization if needed.

### Files Likely Affected

* new gameplay field module
* runtime module definitions
* player/runtime state
* debug overlay / UI

### First-Pass Simplifications

- [ ] Only simulate heat and electrical fields.
- [ ] Use simple shapes and distance checks rather than a fully generalized field framework if that gets to visible results faster.
- [ ] It is acceptable to skip complex dissipation and only model local emitters at first.

### Phase 4 Definition Of Done

* certain modules emit heat or electrical pressure
* the player and modules sample those values at runtime
* the results are visible enough to change behavior

---

## Phase 5 — Local Danger UI And Inspection

### Goal

Make system pressure legible enough to playtest.

### Tasks

- [ ] Add local player readouts for:
  - [ ] heat
  - [ ] electrical danger
- [ ] Add thresholded warnings:
  - [ ] safe
  - [ ] warning
  - [ ] critical
- [ ] Add a component inspection panel showing at least:
  - [ ] module type
  - [ ] integrity
  - [ ] heat
  - [ ] electrical instability
  - [ ] status (healthy / degraded / disabled / destroyed)
- [ ] Add a ship-level alert list or summary panel for major problems.
- [ ] Add a simple way to surface which module currently needs attention most.
- [ ] Optionally add a ship overlay or tinting for overheated / unstable modules.
- [ ] Ensure the UI is visible in both:
  - [ ] direct ship control mode
  - [ ] internal control mode

### Files Likely Affected

* runtime HUD module
* interaction UI
* debug/readability helpers

### First-Pass Simplifications

- [ ] Prefer text and tints over complex custom widgets.
- [ ] Avoid building a giant engineering dashboard for the first pass.
- [ ] Field visualization can start as a debug toggle.

### Phase 5 Definition Of Done

* the player can tell what is in danger
* the player can inspect a nearby component and understand why it is failing
* alerts are readable enough to support decision-making under pressure

---

## Phase 6 — Repair And Stabilization Actions

### Goal

Give the player a meaningful manual response to system pressure.

### Tasks

- [ ] Add a generic repair interaction:
  - [ ] identify damaged / degraded target
  - [ ] hold to repair
  - [ ] restore some integrity and/or clear disabled state
- [ ] Decide whether repairs consume:
  - [ ] nothing in the first pass
  - [ ] abstract repair charge
  - [ ] scrap or another placeholder resource
- [ ] Add a reactor-specific stabilization interaction:
  - [ ] reduce heat
  - [ ] reduce instability
  - [ ] temporarily lower output if needed
- [ ] Add a turret-specific reset or unjam action if turret degradation exists.
- [ ] Add a shield or engine reset interaction if one of those becomes part of the first-pass pressure model.
- [ ] Ensure repair and stabilization are interrupted appropriately by:
  - [ ] moving away
  - [ ] taking excessive danger
  - [ ] mission end
- [ ] Add feedback for successful repair/stabilization:
  - [ ] status clear
  - [ ] warning cleared
  - [ ] module performance restored

### Files Likely Affected

* gameplay interaction module
* runtime module state systems
* HUD feedback

### First-Pass Simplifications

- [ ] Use one generic hold-to-repair duration for most components.
- [ ] It is acceptable for stabilization to be a simple state clear or value reduction rather than a deep minigame.
- [ ] Skip tool/equipment gating for now.

### Phase 6 Definition Of Done

* the player can manually recover a system under pressure
* repair/stabilization changes mission outcomes
* the correct response is sometimes “fix the ship” rather than “keep firing”

---

## Phase 7 — First ARCH Slice

### Goal

Introduce one small, playable piece of automation without implementing the full computer system.

### Tasks

- [ ] Decide on the first ARCH interaction model:
  - [ ] single hardcoded automation script per computer
  - [ ] small preset selector
  - [ ] minimal editable script text box
- [ ] Add one basic computer component to the runtime/editor model if not already present.
- [ ] Define one supported automation target for the first pass, such as:
  - [ ] reactor output regulation on heat threshold
  - [ ] turret auto-disable on overheat
  - [ ] engine cutoff on power deficit
  - [ ] shield mode swap under fire
- [ ] Add a minimal execution/update path for this first automation slice.
- [ ] Add enough UI to:
  - [ ] tell the player what the computer is automating
  - [ ] show whether it is currently active
  - [ ] optionally let the player change or disable it
- [ ] Ensure the automated system reduces one repeated manual task without solving the whole encounter.

### Files Likely Affected

* runtime module state
* new minimal ARCH/gameplay automation module
* editor UI and/or runtime inspection UI

### First-Pass Simplifications

- [ ] Do not build the full instruction parser or register machine yet unless it is already nearly in place.
- [ ] It is acceptable to simulate the first ARCH slice as a configurable behavior module.
- [ ] Limit the first pass to a single automatable problem.

### Phase 7 Definition Of Done

* one small automation feature works in live gameplay
* the player can feel the difference between having it and not having it
* the ship is still mostly manual

---

## Phase 8 — Encounter Pressure Pass

### Goal

Tune encounters so the new systems actually matter.

### Tasks

- [ ] Audit the current hostile encounter and identify where it only creates hull damage rather than system pressure.
- [ ] Adjust enemy damage or behavior so that first-pass pressure is likely to occur:
  - [ ] reactor side getting hot
  - [ ] turret degradation
  - [ ] electrical instability after damage
- [ ] Decide whether certain enemy shots or hazards should bias toward:
  - [ ] heat pressure
  - [ ] electrical interference
- [ ] Add one scenario setup that reliably creates at least one manual intervention moment.
- [ ] Ensure the mission remains readable and not overwhelming while the player is learning internal control.

### Files Likely Affected

* enemy encounter module
* projectile / damage systems
* mission setup/spawn logic

### First-Pass Simplifications

- [ ] Prefer deterministic or heavily guided encounter setups over highly variable chaos.
- [ ] One good “something is going wrong aboard ship” moment is enough.

### Phase 8 Definition Of Done

* the encounter creates pressure that requires repair or stabilization
* internal movement and module interaction matter in a real mission
* the encounter remains beatable and readable

---

## Phase 9 — Return Loop And Editor Feedback

### Goal

Make the return to the editor reflect what happened inside the ship, not just whether scrap was earned.

### Tasks

- [ ] Expand mission report data to record simple system outcomes, such as:
  - [ ] hottest module
  - [ ] first disabled module
  - [ ] number of repairs performed
  - [ ] whether automation was used
- [ ] Surface some of that information in the editor HUD.
- [ ] Add lightweight hints or readouts that support redesign decisions:
  - [ ] reactor placement problem
  - [ ] exposed weapon problem
  - [ ] high-traffic interior problem
- [ ] Ensure progression and module placement UI still remain readable with this added context.

### Files Likely Affected

* mission report resource
* editor HUD
* progression/editor state wiring

### First-Pass Simplifications

- [ ] Use simple text summaries before trying to build rich heatmaps or replay visualizations.
- [ ] It is acceptable for these hints to be observational rather than prescriptive.

### Phase 9 Definition Of Done

* the player returns to the editor with concrete system-level feedback
* ship redesign is motivated by operational problems, not only combat stats

---

## Phase 10 — Usability And Readability Pass

### Goal

Make the systems demo understandable enough to evaluate.

### Tasks

- [ ] Add or refine on-screen controls help for:
  - [ ] switching control modes
  - [ ] moving internally
  - [ ] interacting
  - [ ] repairing
- [ ] Improve prompt clarity and avoid UI overlap between:
  - [ ] runtime ship HUD
  - [ ] internal interaction UI
  - [ ] status alerts
- [ ] Improve visual distinction for:
  - [ ] healthy modules
  - [ ] hot modules
  - [ ] electrically unstable modules
  - [ ] disabled modules
- [ ] Add small polish to make manual intervention feel deliberate:
  - [ ] progress bars
  - [ ] warning clear effects
  - [ ] interaction sounds or flashes if available
- [ ] Do a playtest pass focused on player confusion points and trim complexity where needed.

### Phase 10 Definition Of Done

* a first-time playtester can understand the core loop of onboard intervention
* the new systems are readable enough to judge whether they are fun

---

## Suggested Codebase Expansion

### Near-Term Module Structure

Possible next structure:

* `src/gameplay/`
  * `mod.rs`
  * `state.rs`
  * `spawn.rs`
  * `movement.rs`
  * `power.rs`
  * `combat.rs`
  * `enemy.rs`
  * `salvage.rs`
  * `interior.rs`
  * `interaction.rs`
  * `fields.rs`
  * `status.rs`
  * `repair.rs`
  * `automation.rs`

This does not need to appear all at once, but it is a reasonable direction if the systems demo grows beyond one file.

---

## Priority Order

If we want the fastest route to a meaningful systems demo, do work in this order:

1. Player-on-ship representation
2. Interaction framework
3. Expanded module state
4. Basic field simulation
5. Status / inspection UI
6. Repair and stabilization
7. First ARCH slice
8. Encounter tuning
9. Return-loop feedback
10. Polish/readability

---

## Definition Of Done For Each Stage

### Stage A Done

* the player can exist and move within the ship during runtime

### Stage B Done

* the player can interact with nearby modules

### Stage C Done

* modules can become degraded or unstable without being destroyed

### Stage D Done

* heat and electrical pressure exist and are visible

### Stage E Done

* the player can repair or stabilize a failing system

### Stage F Done

* one limited automation feature works and is useful

### Stage G Done

* encounters reliably create one or more system-pressure moments

### Stage H Done

* returning to the editor reflects system-level lessons learned

---

## Immediate Next Task

The best next implementation task is:

- [ ] Decide and implement the first player-on-ship representation, including how internal position is stored relative to the runtime ship layout

That is the bridge between the current ship-scale demo and the next inner-ship systems slice.
