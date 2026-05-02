# TODO — LUMEN Demo Implementation Breakdown

This file turns `13_LUMEN_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* preserve the distinction between ARCH as direct control and LUMEN as optimization/bias
* make both languages editable and understandable in-game
* keep execution deterministic and bounded
* treat editor/UI work as part of the slice, not a secondary extra
* complete the programmable control stack only after the supporting simulation/UI work exists

## Expected Areas Of Change

Likely touched modules:

* `src/ship/arch.rs`
* `src/ship/mod.rs`
* `src/client/editor/`
* `src/client/gameplay/components/`
* `src/client/gameplay/helpers/`
* `src/client/gameplay/systems/simulation/automation.rs`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/interactions/`
* `docs/src/ARCHLANG.md`
* new LUMEN language/spec docs if needed

Likely new module areas:

* LUMEN program data model
* richer ARCH/LUMEN editor state
* parser/validation helpers
* register-target resolution helpers
* optimization application helpers

---

## Phase 1 — Programming Model Audit

### Goal

Define the finished shape of the direct and optimization language layers.

### Tasks

- [x] Audit the current ARCH interpreter/editor/runtime state.
- [x] Identify gaps between implementation and `ARCHLANG` docs.
- [x] Define the first canonical LUMEN syntax and execution model.
- [x] Decide how ARCH and LUMEN programs coexist on ship systems.

Definition of done:

* the slice has a precise language and runtime target instead of a fuzzy “more automation” goal

---

## Phase 2 — Complete ARCH Instruction Coverage

### Goal

Bring ARCH behavior in line with the intended instruction set.

### Tasks

- [x] Audit every documented ARCH opcode against implementation.
- [x] Add any missing execution semantics.
- [ ] Normalize parsing, operand rules, and validation behavior.
- [x] Ensure domain/error behavior is deterministic and readable.

Definition of done:

* ARCH is functionally complete enough to stop feeling like a partial prototype language

---

## Phase 3 — Canonical Register Surface Pass

### Goal

Make the programmable control surface explicit and coherent.

### Tasks

- [x] Audit readable and writable registers exposed by current component systems.
- [ ] Normalize naming and grouping where needed.
- [x] Ensure runtime components expose the state needed by richer programs.
- [x] Keep manual and automated control surfaces aligned.

Definition of done:

* programmable control has a stable, comprehensible register contract

---

## Phase 4 — Rich ARCH Editor

### Goal

Replace the current basic editing flow with a real in-game engineering tool.

### Tasks

- [x] Add a fuller line-based ARCH editor UI.
- [ ] Support:
  - [x] inserting/removing lines
  - [x] reordering lines
  - [x] editing opcodes
  - [x] editing operands
  - [x] editing constants
  - [ ] naming and saving programs
- [x] Add parse / validation / execution feedback.
- [x] Keep the interface usable without requiring external docs open.

Definition of done:

* players can meaningfully author and debug ARCH in-game

---

## Phase 5 — Runtime ARCH Explainability

### Goal

Make authored automation inspectable during play.

### Tasks

- [x] Show current program and active computer status clearly.
- [x] Show recent writes and halt/error reasons.
- [x] Surface invalid instruction/domain failures cleanly.
- [x] Improve mission/reporting integration for authored automation outcomes.

Definition of done:

* authored automation is something the player can reason about, not a black box

---

## Phase 6 — LUMEN Program Data Model

### Goal

Introduce LUMEN as its own saved language layer.

### Tasks

- [x] Define program/instruction representation for LUMEN.
- [x] Define target specification structures.
- [x] Define optimization operation structures such as:
  - [x] `BUFF`
  - [x] `NERF`
- [x] Add save/load support for LUMEN-authored content.

Definition of done:

* LUMEN exists as a first-class programmable system rather than a vague future note

---

## Phase 7 — LUMEN Syntax And Target Resolution

### Goal

Make LUMEN speak a real language with real targets.

### Tasks

- [ ] Implement first-pass parser/validator for LUMEN syntax.
- [ ] Define target forms such as:
  - [ ] explicit addressed module
  - [x] module-family filter
  - [ ] nearby/range-based groups
  - [ ] register-group targeting
- [x] Resolve those targets deterministically at runtime.
- [x] Surface target resolution in the editor/UI for debugging.

Definition of done:

* LUMEN programs can name and affect real target sets in a readable way

---

## Phase 8 — LUMEN Runtime Semantics

### Goal

Apply optimization/bias behavior without collapsing back into direct control.

### Tasks

- [x] Define how `BUFF`/`NERF` affect target properties.
- [x] Keep effects bounded, deterministic, and inspectable.
- [x] Ensure LUMEN modifies tendencies/weights rather than directly issuing ARCH-style commands.
- [x] Integrate with existing component/system simulation.

Definition of done:

* LUMEN behaves like an optimization layer rather than a second imperative language

---

## Phase 9 — LUMEN Editor And UI

### Goal

Make LUMEN authorable in-game with comparable quality to ARCH.

### Tasks

- [x] Add a dedicated LUMEN editing view/panel.
- [x] Support target selection and optimization-instruction editing.
- [x] Show affected target preview or summary where possible.
- [x] Keep the UI visually distinct from ARCH to reinforce conceptual separation.

Definition of done:

* the player can write optimization logic as a real engineering activity

---

## Phase 10 — ARCH / LUMEN Combined Runtime

### Goal

Make both layers coexist coherently on the same ship.

### Tasks

- [x] Define evaluation order between ARCH and LUMEN.
- [x] Prevent undefined conflicts between direct writes and optimization effects.
- [x] Add clear runtime status reporting when both are active.
- [ ] Ensure mission outcomes can reflect both types of logic separately.

Definition of done:

* direct control and optimization can coexist without conceptual or technical confusion

---

## Phase 11 — Scenario And Content Validation

### Goal

Put the completed language stack under meaningful pressure.

### Tasks

- [ ] Add one or more scenarios where richer ARCH is clearly useful.
- [ ] Add one or more scenarios where LUMEN is clearly useful but not interchangeable with ARCH.
- [x] Ensure the current component roster exposes enough interesting targets.
- [ ] Check that the player can perceive the difference in behavior.

Definition of done:

* both languages justify their existence through play, not only through docs

---

## Phase 12 — Tuning, Docs, And Final Alignment

### Goal

Leave the programming layer coherent both in code and in design docs.

### Tasks

- [ ] Tune instruction budgets, effect magnitudes, and usability.
- [ ] Update docs for final implemented syntax and behavior.
- [ ] Document current limits honestly.
- [x] Ensure save/load, reports, and UI all align with the final model.

Definition of done:

* the project’s automation stack matches its documentation and intended identity

## Current Status

The current implementation now includes:

* a saved LUMEN program model on computer modules
* a dedicated in-editor LUMEN workshop alongside the existing ARCH workshop
* deterministic post-ARCH LUMEN runtime evaluation
* runtime reporting that surfaces both ARCH writes and LUMEN effects

The biggest remaining gaps are:

* a proper text parser/validator for LUMEN syntax rather than button-driven authoring only
* broader target resolution beyond the first family/group-based pass
* fuller docs alignment with the now-implemented runtime/editor model
* scenario/content tuning that proves where LUMEN is useful but not interchangeable with ARCH
