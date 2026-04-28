# TODO — Travel Demo Implementation Breakdown

This file turns `06_TRAVEL_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* keep the current encounter runtime as the combat/salvage core
* add a route-selection outer loop without exploding scope
* make docking the intended place for refit/editing
* keep progression deterministic and saveable
* prefer a small complete loop over a broad but shallow map system

## Expected Areas Of Change

Likely touched modules:

* `src/client/mod.rs`
* `src/client/state.rs`
* `src/client/menu.rs`
* `src/client/editor/`
* `src/client/gameplay/`
* new client modules for:
  * docked UI
  * sector map UI/state
  * encounter selection/progression
* `src/ship/storage.rs` or adjacent save-layer files

---

### Phase 1 — Campaign And Sector State

#### Goal

Define the persistent outer-loop state.

#### Tasks

- [x] Add a persistent campaign/session resource.
- [x] Add `SectorState` with:
  - current node
  - node graph
  - node completion/exhaustion state
  - deterministic seed
- [x] Add `SectorNode` and `SectorNodeKind`.
- [x] Add `EncounterSpec` and `TravelOutcome`.
- [x] Ensure this state can be saved and loaded alongside ship/progression data.

Definition of done:

* the game can describe “where the player is” and “what nearby nodes exist” outside a single encounter

---

### Phase 2 — App Flow Restructure

#### Goal

Restructure the client flow around docked, map, and encounter states.

#### Tasks

- [x] Replace the current top-level loop with:
  - `Docked`
  - `SectorMap`
  - `Encounter`
- [x] Keep `Editing` as a sub-flow entered from `Docked`.
- [x] Rename or repurpose the current `Playing` state into `Encounter`.
- [x] Ensure state transitions are explicit and one-way sensible:
  - `Docked -> SectorMap`
  - `SectorMap -> Encounter`
  - `Encounter -> Docked`
  - `Docked -> Editing -> Docked`

Definition of done:

* the prototype has a real outer-loop state machine

---

### Phase 3 — Hub Station Screen

#### Goal

Create the docked hub as the safe between-mission state.

#### Tasks

- [x] Add a docked station UI screen.
- [x] Show:
  - station title
  - scrap total
  - last mission summary
  - current ship summary
- [x] Add actions:
  - `Refit Ship`
  - `Open Sector Map`
  - optional `Repair Service`
- [x] Make this the standard post-encounter return screen.

Definition of done:

* the player lands in a clear safe hub after each mission

---

### Phase 4 — Sector Map Screen

#### Goal

Create the first route-selection interface.

#### Tasks

- [x] Add a sector map screen for the local node graph.
- [x] Render:
  - current node
  - reachable neighbors
  - node type indicators
  - risk/reward hints
- [x] Add selection and inspection for nodes.
- [x] Add launch/jump only for reachable nodes.
- [x] Show clear return path back to `Docked`.

Definition of done:

* the player can intentionally choose their next destination from a readable small graph

---

### Phase 5 — First Node Graph Generation

#### Goal

Define the first route structure.

#### Tasks

- [x] Generate or author a small deterministic graph:
  - 1 hub station
  - 4–6 encounter nodes
  - 1–2 branches
- [x] Add first node kinds:
  - `HubStation`
  - `SalvageField`
  - `HostileHold`
  - `UnstableDerelict`
- [x] Assign visible risk/reward tags.
- [x] Persist node resolution state across launches.

Definition of done:

* the map contains a small but meaningful local route space

---

### Phase 6 — EncounterSpec Integration

#### Goal

Launch different encounters from different nodes.

#### Tasks

- [x] Refactor runtime encounter bootstrap to accept `EncounterSpec`.
- [x] Parameterize:
  - hostile count/loadout
  - salvage values
  - hazard pressure
  - arena dressing
- [x] Map each node kind to a first-pass encounter profile.
- [x] Ensure encounter runtime can report `TravelOutcome`.

Definition of done:

* encounters feel selected from the map rather than hardcoded globally

---

### Phase 7 — Return And Outcome Application

#### Goal

Make encounter results feed back into the route layer.

#### Tasks

- [x] Apply encounter result on return to `Docked`.
- [x] Update node state:
  - completed
  - exhausted
  - failed/retreated if tracked
- [x] Persist rewards and ship consequences.
- [x] Keep last mission report visible at station.
- [x] Ensure the player’s next map view reflects the resolved node state.

Definition of done:

* completing or failing a node changes the campaign state in a persistent way

---

### Phase 8 — Station-Gated Refit

#### Goal

Turn editing into a docked activity.

#### Tasks

- [x] Make editor entry available only from `Docked` in the player-facing flow.
- [x] Make editor exit return to `Docked`.
- [x] Remove “return to editor from encounter” as the normal user-facing loop.
- [x] Keep an optional debug shortcut only if implementation convenience requires it.

Definition of done:

* refit is clearly part of station life rather than an omnipresent tool

---

### Phase 9 — Optional Station Repair Service

#### Goal

Add one simple non-refit station service.

#### Tasks

- [x] Add a basic repair service button at station.
- [x] Charge scrap for restoring ship integrity/state.
- [x] Keep pricing intentionally simple.
- [x] Surface the decision clearly in the docked UI.

Definition of done:

* station has at least one meaningful service beyond route selection and refit

---

### Phase 10 — Save/Load And Persistence Pass

#### Goal

Ensure the outer loop survives restart cleanly.

#### Tasks

- [x] Save/load sector graph seed and node states.
- [x] Save/load current docked node.
- [x] Save/load scrap/progression and ship definition together.
- [x] Confirm mission return and station screen restore correctly after restart.

Definition of done:

* the travel loop is persistent and restart-safe

---

### Phase 11 — UX And Readability Pass

#### Goal

Make the outer loop understandable without extra explanation.

#### Tasks

- [x] Improve docked and sector-map labels.
- [x] Make node type/risk/reward hints easy to scan.
- [x] Add clear transition copy between:
  - docked
  - route selection
  - encounter
- [x] Keep screen structure distinct so modes do not blur together.

Definition of done:

* first-time players can understand where they are in the loop and what action comes next

---

### Phase 12 — Playtest And Tuning

#### Goal

Tune the route loop until the choice structure feels real.

#### Tasks

- [ ] Tune node rewards vs threat.
- [ ] Tune number of nodes and branch depth.
- [ ] Check whether station-gated refit improves the game feel.
- [ ] Check whether players understand why to choose one node over another.
- [ ] Verify the loop feels like “campaign progress” rather than just menu hopping.

Definition of done:

* the game feels meaningfully closer to the original concept’s travel-engage-salvage-repeat loop

### Immediate Next Task

Implementation status:

* phases 1 through 11 are now in place in the prototype
* phase 12 remains intentionally open for live playtest/tuning rather than code-only completion

## Assumptions

* no full economy in this slice
* no misjump/in-transit event system yet
* no full procedural galaxy
* node-based route selection is the first proof target
* station docking is the intended refit/edit anchor from this slice onward
