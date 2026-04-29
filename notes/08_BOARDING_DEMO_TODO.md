# TODO — Boarding Demo Implementation Breakdown

This file turns `08_BOARDING_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* preserve the current travel and encounter loop
* improve the runtime through continuous player embodiment rather than replacing core systems
* reuse existing component/station logic on hostile ships wherever possible
* prefer systemic boarding over bespoke mission scripting
* keep the first cargo carry loop intentionally small and legible

## Expected Areas Of Change

Likely touched modules:

* `src/client/mod.rs`
* `src/client/state.rs`
* `src/client/gameplay/components/`
* `src/client/gameplay/spawn/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/interactions/`
* `src/client/gameplay/systems/simulation/`
* `src/client/gameplay/systems/ui/`
* `src/client/sector_map.rs`
* `src/client/docked.rs`

## Current Status

Phases 1 through 11 are implemented in first-pass form.

Notable first-pass simplifications:

* ship entry is driven by inertia-field attachment and nearby-station access rather than strict airlock door gating
* player traversal is continuous, but still intentionally lightweight rather than full collision-heavy character simulation
* boarding rewards are centered on carried cargo extraction and system interference using existing controls

---

## Phase 1 — Continuous Player Actor State

### Goal

Replace discrete onboard locomotion with a continuous player actor model.

### Tasks

- [x] Add continuous player position and velocity components/resources.
- [x] Separate station focus from locomotion position.
- [x] Keep nearby interaction detection working against continuous position.
- [x] Preserve the current shipboard marker and focus model during the migration.

Definition of done:

* the player is represented as a continuous-moving runtime actor rather than a node-hopping cursor

---

## Phase 2 — Continuous Movement Controls

### Goal

Make player motion continuous both aboard ship and in EVA.

### Tasks

- [x] Replace discrete `WASD` node stepping with continuous directional thrust/motion.
- [x] Add damping / drift behavior appropriate to player movement mode.
- [x] Keep station interaction range-based instead of tile-based.
- [x] Ensure the player marker follows continuous motion correctly.

Definition of done:

* player motion is visibly continuous and no longer snaps tile-by-tile

---

## Phase 3 — Reference Frames And Inertia Fields

### Goal

Define how the player transitions between world-relative and ship-relative motion.

### Tasks

- [x] Add inertia field data to runtime ships.
- [x] Add player reference-frame state:
  - `World`
  - `ShipLocal`
- [x] Detect field entry/exit against player position.
- [x] Convert player motion appropriately when entering/leaving fields.
- [x] Ensure only one dominant field/frame is active at a time.

Definition of done:

* the player cleanly transitions between EVA and ship-relative traversal

---

## Phase 4 — Camera Frame Switching

### Goal

Make the camera reflect the player’s current frame of reference.

### Tasks

- [x] Use world-relative camera behavior in EVA.
- [x] Use ship-relative rotating camera behavior inside a ship field.
- [x] Smooth frame changes so transitions do not feel abrupt.
- [x] Keep cockpit/turret camera behavior intact when focusing those stations.

Definition of done:

* the camera clearly communicates whether the player is in space or aboard a ship

---

## Phase 5 — Boarding-Capable Hostile Ships In Real Sector Nodes

### Goal

Ensure ordinary encounters actually spawn boardable ships.

### Tasks

- [x] Audit current sector nodes and replace turret-only nodes where appropriate.
- [x] Ensure at least several nodes spawn real enemy ships by default.
- [x] Keep test-range and simple encounters where useful for debug/playtest.
- [x] Tune encounter profiles so hostile ships can be approached and boarded.

Definition of done:

* boarding opportunities appear in ordinary travel flow

---

## Phase 6 — Hostile Ship Interior Access

### Goal

Let the player physically enter hostile ships.

### Tasks

- [x] Allow the player to cross from EVA into hostile ship inertia fields.
- [x] Define first-pass access behavior around hostile airlocks/interior.
- [x] Ensure hostile ship interior traversal uses the same movement model as the player ship.
- [x] Preserve nearby interaction and station focus on hostile modules.

Definition of done:

* the player can physically board an enemy ship and move around inside it

---

## Phase 7 — Reuse Existing System Interaction On Hostile Ships

### Goal

Treat enemy ships as real ships, not special objective maps.

### Tasks

- [x] Allow reactor controls on hostile ships.
- [x] Allow computer/ARCH interaction on hostile ships.
- [x] Allow logistics/storage inspection on hostile ships.
- [x] Allow sabotage through existing interaction pathways:
  - disable systems
  - shut down reactors
  - interfere with computers
- [x] Ensure UI panels identify whether the player is using allied or hostile equipment.

Definition of done:

* boarding gameplay comes from real system interaction rather than bespoke mission prompts

---

## Phase 8 — Physical Carryable Cargo

### Goal

Introduce carried items as a first-class extraction mechanic.

### Tasks

- [x] Add carryable cargo entities or payload state.
- [x] Allow pickup from space and hostile ship storage/output points.
- [x] Limit the first pass to a simple carrying model:
  - one carried item at a time
  - explicit drop/deposit behavior
- [x] Visually represent when the player is carrying cargo.

Definition of done:

* the player can physically carry loot rather than only triggering abstract salvage gain

---

## Phase 9 — Deposit Back Into Player Ship Storage

### Goal

Close the extraction loop by bringing cargo home.

### Tasks

- [x] Allow carried cargo to be deposited into player ship airlock/storage.
- [x] Feed deposited cargo into existing logistics/storage systems.
- [x] Ensure deposited cargo is reflected in mission/reporting state.
- [x] Keep the first implementation robust even if logistics automation is disabled.

Definition of done:

* cargo can move from hostile ship or space into the player ship’s real storage chain

---

## Phase 10 — EVA / Boarding HUD And Readability

### Goal

Make the new traversal state easy to understand.

### Tasks

- [x] Show whether the player is in EVA or ship-relative mode.
- [x] Show current ship frame when inside an inertia field.
- [x] Show carried cargo state.
- [x] Improve boarding interaction prompts around hostile modules.
- [x] Keep cockpit/turret/station UI legible during frame transitions.

Definition of done:

* players can tell where they are, what they are attached to, and what they are carrying

---

## Phase 11 — Return-Loop Integration

### Goal

Make boarding results matter after the encounter.

### Tasks

- [x] Track extracted cargo separately from purely destroyed salvage.
- [x] Reflect boarded/sabotaged outcomes in the mission report.
- [x] Feed extracted goods into scrap / logistics reward flow.
- [x] Add redesign hints based on what the player chose to manipulate aboard hostile ships.

Definition of done:

* the post-mission loop understands the difference between boarding extraction and simple destruction

---

## Phase 12 — Tuning And Stability Pass

### Goal

Tune the new traversal and boarding loop into something readable and reliable.

### Tasks

- [ ] Tune EVA movement feel.
- [ ] Tune inertia field sizes and transitions.
- [ ] Tune hostile ship encounter frequency in sector nodes.
- [ ] Tune carried cargo friction so extraction is meaningful but not tedious.
- [ ] Check that boarding remains compatible with current deterministic/runtime architecture.

Definition of done:

* boarding feels like a real extension of the game rather than a prototype-only gimmick

## Immediate Next Task

Continue with **Phase 12**:

* tune EVA acceleration and damping
* tune inertia-field radius and transition feel
* tune how often sector encounters surface boardable ships
* verify carrying cargo home feels meaningful without becoming busywork

That is the remaining work before the slice can be called fully tuned.

## Assumptions

* no hostile humanoid crew for this slice
* no ship capture as ownership transfer yet
* cargo carrying starts simple and constrained
* enemy ships should use the same systemic affordances as player ships wherever practical
* EVA and inertia-field behavior are core to this slice, not optional polish
