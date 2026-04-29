# TODO — Breach Demo Implementation Breakdown

This file turns `09_BREACH_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* preserve the existing travel, combat, boarding, and cargo loops
* keep atmosphere deterministic and ship-local
* prefer readable systemic pressure over high-fidelity simulation
* make player and hostile ships obey the same rules
* keep the first oxygen model simple enough to tune

## Expected Areas Of Change

Likely touched modules:

* `src/client/mod.rs`
* `src/client/state.rs`
* `src/client/gameplay/components/`
* `src/client/gameplay/helpers/`
* `src/client/gameplay/spawn/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/simulation/`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/interactions/`

## Current Status

Phases 1 through 10 are implemented in first-pass form.

Notable first-pass simplifications:

* oxygen is the only atmosphere quantity
* venting is driven by open airlocks and destroyed exterior tiles rather than a full pressure sim
* low oxygen currently creates strong readability and movement-pressure rather than a full survival-failure loop

---

## Phase 1 — Ship Atmosphere Data Model

### Goal

Define the runtime atmosphere representation.

### Tasks

- [x] Add ship-local atmosphere tile state.
- [x] Add tile oxygen quantity storage.
- [x] Add sealing / breach / exposed-boundary markers or derived state.
- [x] Attach atmosphere data to both player and hostile ships.

Definition of done:

* each runtime ship can represent oxygen and exposure on interior tiles

---

## Phase 2 — Interior Tile And Boundary Mapping

### Goal

Derive the tiles and boundaries needed for atmosphere simulation.

### Tasks

- [x] Reuse ship layout to build interior atmosphere tiles.
- [x] Identify exterior-facing boundaries.
- [x] Identify first-pass sealed vs open tiles.
- [x] Include airlock-driven openings in the derived state.

Definition of done:

* atmosphere simulation knows where space can leak in or out

---

## Phase 3 — Oxygen Initialization

### Goal

Seed ships with usable onboard atmosphere.

### Tasks

- [x] Initialize player ship interior oxygen.
- [x] Initialize hostile ship interior oxygen.
- [x] Keep oxygen stable in sealed ships at encounter start.
- [x] Ensure test/debug nodes produce sensible starting states.

Definition of done:

* ships begin encounters with meaningful onboard atmosphere

---

## Phase 4 — Venting And Equalization

### Goal

Simulate first-pass oxygen loss and redistribution.

### Tasks

- [x] Add per-tick oxygen equalization between connected tiles.
- [x] Add oxygen loss through exposed boundaries.
- [x] Make venting depend on open airlocks and breaches.
- [x] Keep the update deterministic and numerically stable.

Definition of done:

* opened ships lose oxygen and sealed compartments retain it

---

## Phase 5 — Airlock And Breach Integration

### Goal

Connect atmosphere changes to existing ship interactions and combat.

### Tasks

- [x] Make airlock state affect atmosphere boundaries.
- [x] Add first-pass hull breach state from combat damage or destroyed edge modules.
- [x] Ensure breaches on hostile ships affect boarding conditions.
- [x] Ensure player ship breaches create meaningful stabilization pressure.

Definition of done:

* atmosphere responds to real ship openings rather than only synthetic debug state

---

## Phase 6 — Player Oxygen Sampling And Hazard

### Goal

Make atmosphere matter to the embodied player.

### Tasks

- [x] Sample local tile oxygen for the player.
- [x] Distinguish EVA vacuum from low-oxygen interior state.
- [x] Add first-pass danger accumulation or performance penalty.
- [x] Ensure hazards are readable but not instantly fatal.

Definition of done:

* the player can feel and understand unsafe atmosphere

---

## Phase 7 — Hostile Ship Atmosphere Exploitation

### Goal

Make decompression tactically useful during boarding.

### Tasks

- [x] Ensure hostile ship atmosphere can be reduced by breaches or airlocks.
- [x] Make hostile boarding conditions visibly change when a ship is vented.
- [x] Allow existing boarding/system interactions to remain functional under atmosphere pressure.
- [x] Keep this systemic rather than adding bespoke boarding objectives.

Definition of done:

* decompression becomes part of the boarding decision space

---

## Phase 8 — HUD And Inspection Readability

### Goal

Surface atmosphere state clearly to the player.

### Tasks

- [x] Show local oxygen on the runtime HUD.
- [x] Show whether the player is in EVA, breathable space, or low atmosphere.
- [x] Show compartment or station oxygen state where helpful.
- [x] Improve prompts around airlocks, breaches, and unsafe boarding spaces.

Definition of done:

* the player can understand atmosphere state without guesswork

---

## Phase 9 — Mission Outcome And Report Integration

### Goal

Make atmosphere management matter after the encounter.

### Tasks

- [x] Track major venting / stabilization outcomes in mission state.
- [x] Reflect decompression outcomes in the return report.
- [x] Add redesign hints for poor compartmenting or breach control.
- [x] Ensure extraction outcomes can differ depending on atmosphere management.

Definition of done:

* atmosphere has visible consequences in the post-mission loop

---

## Phase 10 — ARCH Compatibility Hooks

### Goal

Keep the atmosphere model aligned with future automation.

### Tasks

- [x] Identify first writable/readable control surfaces for later ARCH use.
- [x] Keep airlock and seal-related state deterministic and inspectable.
- [x] Avoid hardcoding manual-only pathways that would block later automation.
- [x] Add minimal notes or scaffolding where needed.

Definition of done:

* the slice does not paint future ARCH/environment control into a corner

---

## Phase 11 — Tuning And Stability Pass

### Goal

Make decompression readable, fair, and worth using.

### Tasks

- [ ] Tune oxygen depletion and equalization rates.
- [ ] Tune breach severity and airlock impact.
- [ ] Tune player danger timing.
- [ ] Check that boarding remains fun rather than overly punitive.
- [ ] Verify the sim behaves reliably on both player and hostile ships.

Definition of done:

* atmosphere feels like a real extension of the game, not a prototype-only nuisance

## Immediate Next Task

Continue with **Phase 11**:

* tune oxygen equalization and venting rates
* tune low-oxygen movement pressure
* verify boarding remains fun when hostile ships are partially vented
* tune when decompression becomes a useful tactic versus pure nuisance

That is the remaining work before the slice can be called fully tuned.

## Assumptions

* oxygen is the first and only atmosphere quantity in this slice
* the first pass uses a simplified equalization/leak model
* hostile ships follow the same atmosphere rules as player ships
* airlocks and breaches are the main ways atmosphere changes
* full life-support production and advanced gas mechanics are deferred
